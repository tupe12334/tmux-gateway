mod graphql;
mod grpc;
mod rest;

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("tmux_gateway=info".parse().unwrap()),
        )
        .init();

    let http_app = axum::Router::new()
        .merge(rest::router())
        .merge(graphql::router())
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", rest::ApiDoc::openapi()),
        );

    let http_handle = tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
        tracing::info!("HTTP server (REST + GraphQL + Swagger) listening on 0.0.0.0:3000");
        axum::serve(listener, http_app).await.unwrap();
    });

    let grpc_handle = tokio::spawn(async move {
        let addr = "0.0.0.0:50051".parse().unwrap();
        tracing::info!("gRPC server listening on {}", addr);
        tonic::transport::Server::builder()
            .add_service(grpc::health_server())
            .serve(addr)
            .await
            .unwrap();
    });

    tokio::try_join!(http_handle, grpc_handle).unwrap();
}
