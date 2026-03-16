use crate::api::{grpc, middleware};
use anyhow::Context;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};

use crate::transports::shutdown::remove_stale_socket;

pub async fn spawn(
    socket_path: &str,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<JoinHandle<()>> {
    remove_stale_socket(socket_path);
    let uds = tokio::net::UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind gRPC Unix socket at {socket_path}"))?;
    tracing::info!("gRPC Unix socket listening on {socket_path}");
    let incoming = tokio_stream::wrappers::UnixListenerStream::new(uds);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_file_descriptor_set(grpc::file_descriptor_set())
        .build_v1()
        .context("failed to build gRPC reflection service (unix)")?;

    let (_, health_service) = tonic_health::server::health_reporter();
    let x_request_id = http::HeaderName::from_static("x-request-id");

    Ok(tokio::spawn(async move {
        if let Err(e) = tonic::transport::Server::builder()
            .layer(
                tower::ServiceBuilder::new()
                    .layer(SetRequestIdLayer::new(
                        x_request_id.clone(),
                        middleware::UuidRequestId,
                    ))
                    .layer(PropagateRequestIdLayer::new(x_request_id))
                    .into_inner(),
            )
            .add_service(health_service)
            .add_service(grpc::grpc_server())
            .add_service(reflection_service)
            .serve_with_incoming_shutdown(incoming, async move {
                let _ = shutdown_rx.wait_for(|&v| v).await;
                tracing::info!("gRPC Unix socket server shutting down...");
            })
            .await
        {
            tracing::error!("gRPC Unix socket server error: {e:#}");
        }
    }))
}
