use crate::api::{grpc, middleware};
use anyhow::Context;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;

pub async fn spawn(
    port: u16,
    mut shutdown_rx: watch::Receiver<bool>,
    mut health_shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<JoinHandle<()>> {
    let grpc_addr = format!("0.0.0.0:{port}");
    let grpc_listener = TcpListener::bind(&grpc_addr)
        .await
        .with_context(|| format!("failed to bind gRPC port {port} — port may already be in use"))?;
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_file_descriptor_set(grpc::file_descriptor_set())
        .build_v1()
        .context("failed to build gRPC reflection service")?;

    Ok(tokio::spawn(async move {
        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        // Spawn a background task that periodically verifies tmux is responsive
        // and updates the gRPC health status accordingly.
        {
            let reporter = health_reporter.clone();
            tokio::spawn(async move {
                const CHECK_INTERVAL: Duration = Duration::from_secs(5);
                const CHECK_TIMEOUT: Duration = Duration::from_secs(3);

                loop {
                    let healthy =
                        tokio::time::timeout(
                            CHECK_TIMEOUT,
                            tmux_gateway_core::is_available(
                                &tmux_gateway_core::RealTmuxExecutor::new(),
                            ),
                        )
                        .await
                        .unwrap_or(false);

                    if healthy {
                        reporter
                            .set_serving::<grpc::TmuxGatewayServerConcrete>()
                            .await;
                    } else {
                        reporter
                            .set_not_serving::<grpc::TmuxGatewayServerConcrete>()
                            .await;
                    }

                    tokio::select! {
                        () = tokio::time::sleep(CHECK_INTERVAL) => {}
                        _ = health_shutdown_rx.wait_for(|&v| v) => break,
                    }
                }

                tracing::info!("Health check loop stopped");
            });
        }

        let x_request_id = http::HeaderName::from_static("x-request-id");
        let incoming = tokio_stream::wrappers::TcpListenerStream::new(grpc_listener);

        tracing::info!("gRPC server listening on {grpc_addr}");
        if let Err(e) = tonic::transport::Server::builder()
            .layer(
                tower::ServiceBuilder::new()
                    .layer(SetRequestIdLayer::new(
                        x_request_id.clone(),
                        middleware::UuidRequestId,
                    ))
                    .layer(
                        TraceLayer::new_for_grpc()
                            .make_span_with(|request: &http::Request<_>| {
                                let request_id = request
                                    .headers()
                                    .get("x-request-id")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("-");
                                tracing::info_span!(
                                    "grpc_request",
                                    method = %request.uri().path(),
                                    request_id = %request_id,
                                )
                            })
                            .on_response(
                                |response: &http::Response<_>, latency: Duration, _span: &Span| {
                                    tracing::info!(
                                        status = response.status().as_u16(),
                                        latency_ms = latency.as_millis(),
                                        "response"
                                    );
                                },
                            ),
                    )
                    .layer(PropagateRequestIdLayer::new(x_request_id))
                    .into_inner(),
            )
            .add_service(health_service)
            .add_service(grpc::grpc_server())
            .add_service(reflection_service)
            .serve_with_incoming_shutdown(incoming, async move {
                let _ = shutdown_rx.wait_for(|&v| v).await;
                tracing::info!("gRPC server shutting down...");
            })
            .await
        {
            tracing::error!("gRPC server error: {e:#}");
        }
    }))
}
