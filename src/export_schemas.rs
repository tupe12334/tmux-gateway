use std::fs;
use std::path::Path;

use async_graphql::SDLExportOptions;
use utoipa::OpenApi;

use crate::api::{graphql, grpc, rest};

pub fn openapi_json() -> String {
    let openapi = rest::ApiDoc::openapi();
    serde_json::to_string_pretty(&openapi).expect("Failed to serialize OpenAPI spec")
}

pub fn graphql_sdl() -> String {
    let schema = graphql::build_schema();
    schema.sdl_with_options(SDLExportOptions::new())
}

pub fn export_all() {
    let schemas_dir = Path::new("schemas");
    fs::create_dir_all(schemas_dir).expect("Failed to create schemas directory");

    // Export OpenAPI JSON
    let openapi = rest::ApiDoc::openapi();
    let openapi_json =
        serde_json::to_string_pretty(&openapi).expect("Failed to serialize OpenAPI spec");
    fs::write(schemas_dir.join("openapi.json"), openapi_json)
        .expect("Failed to write openapi.json");
    tracing::info!("Exported schemas/openapi.json");

    // Export GraphQL SDL
    let schema = graphql::build_schema();
    let sdl = schema.sdl_with_options(SDLExportOptions::new());
    fs::write(schemas_dir.join("schema.graphql"), sdl).expect("Failed to write schema.graphql");
    tracing::info!("Exported schemas/schema.graphql");

    // Export gRPC proto schema
    fs::write(
        schemas_dir.join("tmux_gateway.proto"),
        grpc::proto_content(),
    )
    .expect("Failed to write tmux_gateway.proto");
    tracing::info!("Exported schemas/tmux_gateway.proto");

    // Export gRPC file descriptor set
    let fds = grpc::file_descriptor_set();
    let fds_bytes = prost::Message::encode_to_vec(&fds);
    fs::write(schemas_dir.join("tmux_gateway_descriptor.bin"), fds_bytes)
        .expect("Failed to write tmux_gateway_descriptor.bin");
    tracing::info!("Exported schemas/tmux_gateway_descriptor.bin");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_is_valid_json() {
        let json_str = openapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["info"]["title"], "tmux-gateway");
        assert_eq!(v["openapi"].as_str().unwrap().split('.').next().unwrap(), "3");
    }

    #[test]
    fn openapi_has_all_paths() {
        let json_str = openapi_json();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let paths = v["paths"].as_object().unwrap();
        let expected = [
            "/health", "/ls", "/new", "/kill-session", "/kill-window", "/kill-pane",
            "/list-windows", "/list-panes", "/send-keys", "/rename-session",
            "/rename-window", "/new-window", "/split-window", "/capture-pane",
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
    }

    #[test]
    fn graphql_sdl_has_operations() {
        let sdl = graphql_sdl();
        let expected = [
            "health", "ls", "createSession", "killSession", "killWindow", "killPane",
            "listWindows", "listPanes", "sendKeys", "renameSession", "renameWindow",
            "newWindow", "splitWindow", "capturePane",
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
}
