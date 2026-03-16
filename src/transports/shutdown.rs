use std::path::Path;
use tokio::signal;

/// Waits for a shutdown signal (Ctrl+C or SIGTERM).
pub async fn shutdown_signal() {
    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.expect("failed to listen for Ctrl+C");
    }
}

/// Removes a stale Unix socket file if it exists, so we can bind a fresh one.
pub fn remove_stale_socket(path: &str) {
    let p = Path::new(path);
    if p.exists() {
        tracing::info!(path, "Removing stale Unix socket file");
        let _ = std::fs::remove_file(p);
    }
}
