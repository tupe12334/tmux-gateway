use std::fs;
use std::path::Path;

use async_graphql::SDLExportOptions;
use utoipa::OpenApi;

use crate::api::{graphql, grpc, rest};

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
    fs::write(schemas_dir.join("tmux_gateway.proto"), grpc::proto_content())
        .expect("Failed to write tmux_gateway.proto");
    tracing::info!("Exported schemas/tmux_gateway.proto");

    // Export gRPC file descriptor set
    let fds = grpc::file_descriptor_set();
    let fds_bytes = prost::Message::encode_to_vec(&fds);
    fs::write(
        schemas_dir.join("tmux_gateway_descriptor.bin"),
        fds_bytes,
    )
    .expect("Failed to write tmux_gateway_descriptor.bin");
    tracing::info!("Exported schemas/tmux_gateway_descriptor.bin");
}
