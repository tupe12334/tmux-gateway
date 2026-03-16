use std::fs;
use std::path::Path;

use anyhow::Context;
use async_graphql::SDLExportOptions;
use utoipa::OpenApi;

use crate::api::{graphql, grpc, rest};
use crate::asyncapi;
use crate::tmux_docs;

pub fn openapi_json() -> String {
    let openapi = rest::ApiDoc::openapi();
    let mut value =
        serde_json::to_value(&openapi).expect("Failed to convert OpenAPI spec to Value");
    tmux_docs::enrich_openapi(&mut value);
    serde_json::to_string_pretty(&value).expect("Failed to serialize OpenAPI spec")
}

pub fn graphql_sdl() -> String {
    let schema = graphql::build_schema();
    schema.sdl_with_options(SDLExportOptions::new())
}

pub fn asyncapi_json() -> String {
    let spec = asyncapi::asyncapi_spec();
    serde_json::to_string_pretty(&spec).expect("Failed to serialize AsyncAPI spec")
}

pub fn export_all() {
    if let Err(e) = try_export_all() {
        tracing::error!("Schema export failed: {e:#}");
    }
}

fn try_export_all() -> anyhow::Result<()> {
    let schemas_dir = Path::new("schemas");
    fs::create_dir_all(schemas_dir).context("failed to create schemas directory")?;

    // Export OpenAPI JSON (enriched with tmux doc links)
    let openapi = rest::ApiDoc::openapi();
    let mut openapi_value =
        serde_json::to_value(&openapi).context("failed to convert OpenAPI spec to Value")?;
    tmux_docs::enrich_openapi(&mut openapi_value);
    let openapi_json =
        serde_json::to_string_pretty(&openapi_value).context("failed to serialize OpenAPI spec")?;
    fs::write(schemas_dir.join("openapi.json"), openapi_json)
        .context("failed to write openapi.json")?;
    tracing::info!("Exported schemas/openapi.json");

    // Export GraphQL SDL
    let schema = graphql::build_schema();
    let sdl = schema.sdl_with_options(SDLExportOptions::new());
    fs::write(schemas_dir.join("schema.graphql"), sdl).context("failed to write schema.graphql")?;
    tracing::info!("Exported schemas/schema.graphql");

    // Export gRPC proto schema (enriched with tmux doc links)
    let proto = tmux_docs::enrich_proto(&grpc::proto_content());
    fs::write(schemas_dir.join("tmux_gateway.proto"), &proto)
        .context("failed to write tmux_gateway.proto")?;
    tracing::info!("Exported schemas/tmux_gateway.proto");

    // Export gRPC file descriptor set
    let fds = grpc::file_descriptor_set();
    let fds_bytes = prost::Message::encode_to_vec(&fds);
    fs::write(schemas_dir.join("tmux_gateway_descriptor.bin"), fds_bytes)
        .context("failed to write tmux_gateway_descriptor.bin")?;
    tracing::info!("Exported schemas/tmux_gateway_descriptor.bin");

    // Export AsyncAPI specification
    let asyncapi = asyncapi_json();
    fs::write(schemas_dir.join("asyncapi.json"), asyncapi)
        .context("failed to write asyncapi.json")?;
    tracing::info!("Exported schemas/asyncapi.json");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_is_valid_json() {
        let json_str = openapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["info"]["title"], "tmux-gateway");
        assert_eq!(
            v["openapi"].as_str().unwrap().split('.').next().unwrap(),
            "3"
        );
    }

    #[test]
    fn openapi_has_all_paths() {
        let json_str = openapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let paths = v["paths"].as_object().unwrap();
        let expected = [
            "/health",
            "/ls",
            "/new",
            "/kill-session",
            "/kill-window",
            "/kill-pane",
            "/list-windows",
            "/list-panes",
            "/send-keys",
            "/rename-session",
            "/rename-window",
            "/new-window",
            "/split-window",
            "/capture-pane",
        ];
        for path in &expected {
            assert!(paths.contains_key(*path), "missing path: {path}");
        }
    }

    #[test]
    fn openapi_has_schemas() {
        let json_str = openapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let schemas = v["components"]["schemas"].as_object().unwrap();
        assert!(schemas.contains_key("HealthResponse"));
        assert!(schemas.contains_key("SessionResponse"));
        assert!(schemas.contains_key("NewSessionRequest"));
        assert!(schemas.contains_key("KillTargetRequest"));
    }

    #[test]
    fn graphql_sdl_has_types() {
        let sdl = graphql_sdl();
        assert!(sdl.contains("type Query"));
        assert!(sdl.contains("type Mutation"));
        assert!(sdl.contains("type Subscription"));
    }

    #[test]
    fn graphql_sdl_has_operations() {
        let sdl = graphql_sdl();
        let expected = [
            "health",
            "ls",
            "createSession",
            "killSession",
            "killWindow",
            "killPane",
            "listWindows",
            "listPanes",
            "sendKeys",
            "renameSession",
            "renameWindow",
            "newWindow",
            "splitWindow",
            "capturePane",
            "paneOutput",
        ];
        for op in &expected {
            assert!(sdl.contains(op), "missing GraphQL operation: {op}");
        }
    }

    #[test]
    fn graphql_sdl_has_session_type() {
        let sdl = graphql_sdl();
        assert!(sdl.contains("Session"));
        assert!(sdl.contains("name"));
        assert!(sdl.contains("windows"));
        assert!(sdl.contains("attached"));
    }

    #[test]
    fn asyncapi_is_valid_json() {
        let json_str = asyncapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["asyncapi"], "3.0.0");
        assert_eq!(v["info"]["title"], "tmux-gateway");
    }

    #[test]
    fn asyncapi_has_all_channels() {
        let json_str = asyncapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let channels = v["channels"].as_object().unwrap();
        assert!(channels.contains_key("wsPaneOutput"));
        assert!(channels.contains_key("grpcStreamPaneOutput"));
        assert!(channels.contains_key("graphqlPaneOutput"));
    }
}
