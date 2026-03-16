use anyhow::Context;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::transports::shutdown::remove_stale_socket;

pub async fn spawn(
    app: axum::Router,
    socket_path: &str,
    mut shutdown_rx: watch::Receiver<bool>,
) -> anyhow::Result<JoinHandle<()>> {
    remove_stale_socket(socket_path);
    let uds = tokio::net::UnixListener::bind(socket_path)
        .with_context(|| format!("failed to bind HTTP Unix socket at {socket_path}"))?;
    tracing::info!("HTTP Unix socket listening on {socket_path}");

    Ok(tokio::spawn(async move {
        if let Err(e) = axum::serve(uds, app.into_make_service())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.wait_for(|&v| v).await;
                tracing::info!("HTTP Unix socket server shutting down...");
            })
            .await
        {
            tracing::error!("HTTP Unix socket server error: {e:#}");
        }
    }))
}
