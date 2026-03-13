mod graphql;
mod grpc;
mod rest;

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("tmux_gateway=info".parse().unwrap()),
        )
        .init();

    let http_app = axum::Router::new()
        .merge(rest::router())
        .merge(graphql::router());

    let http_handle = tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        tracing::info!("HTTP server (REST + GraphQL) listening on 0.0.0.0:8080");
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
