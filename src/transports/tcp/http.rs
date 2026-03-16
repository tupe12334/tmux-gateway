use anyhow::Context;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub async fn spawn(
    app: axum::Router,
    port: u16,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<JoinHandle<()>> {
    let http_addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&http_addr)
        .await
        .with_context(|| format!("failed to bind HTTP port {port} — port may already be in use"))?;
    tracing::info!("HTTP server (REST + GraphQL + Swagger) listening on {http_addr}");

    Ok(tokio::spawn(async move {
        if let Err(e) = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.wait_for(|&v| v).await;
            tracing::info!("HTTP server shutting down...");
        })
        .await
        {
            tracing::error!("HTTP server error: {e:#}");
        }
    }))
}
