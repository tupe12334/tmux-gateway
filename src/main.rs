use std::env;

use tmux_gateway::{export_schemas, graphql, grpc, port_table, rest};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("tmux_gateway=info".parse().unwrap()),
        )
        .init();

    export_schemas::export_all();

    let http_port = env::var("HTTP_PORT").expect("HTTP_PORT must be set");
    let grpc_port = env::var("GRPC_PORT").expect("GRPC_PORT must be set");

    let http_port_num: u16 = http_port.parse().expect("HTTP_PORT must be a number");
    let grpc_port_num: u16 = grpc_port.parse().expect("GRPC_PORT must be a number");

    let swagger_url = format!("http://localhost:{}/swagger-ui", http_port_num);
    let graphql_url = format!("http://localhost:{}/graphql", http_port_num);
    let grpcui_cmd = format!("grpcui -plaintext localhost:{}", grpc_port_num);

    port_table::print_port_table(&[
        ("REST", http_port_num, swagger_url.as_str()),
        ("GraphQL", http_port_num, graphql_url.as_str()),
        ("gRPC", grpc_port_num, grpcui_cmd.as_str()),
    ]);

    let http_app = axum::Router::new()
        .merge(rest::router())
        .merge(graphql::router())
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", rest::ApiDoc::openapi()),
        );

    let http_addr = format!("0.0.0.0:{http_port_num}");
    let http_handle = tokio::spawn(async move {
        let listener = TcpListener::bind(&http_addr).await.unwrap();
        tracing::info!("HTTP server (REST + GraphQL + Swagger) listening on {http_addr}");
        axum::serve(listener, http_app).await.unwrap();
    });

    let grpc_addr = format!("0.0.0.0:{grpc_port_num}");
    let grpc_handle = tokio::spawn(async move {
        let addr = grpc_addr.parse().unwrap();
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(grpc::FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();
        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter
            .set_serving::<grpc::TmuxGatewayServer>()
            .await;
        tracing::info!("gRPC server listening on {}", addr);
        tonic::transport::Server::builder()
            .add_service(health_service)
            .add_service(grpc::grpc_server())
            .add_service(reflection_service)
            .serve(addr)
            .await
            .unwrap();
    });

    tokio::try_join!(http_handle, grpc_handle).unwrap();
}
