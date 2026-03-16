mod common;

use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::watch;

/// Verify that the HTTP server shuts down gracefully when notified
/// via the watch channel (simulating SIGTERM/SIGINT behavior).
#[tokio::test]
async fn http_server_shuts_down_on_watch_signal() {
    common::require_tmux();

    let (shutdown_tx, _) = watch::channel(false);

    let app = tmux_gateway::api::rest::router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let mut shutdown_rx = shutdown_tx.subscribe();
    let server_handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.wait_for(|&v| v).await;
        })
        .await
        .unwrap();
    });

    // Verify server is accepting connections
    tokio::net::TcpStream::connect(addr).await.unwrap();

    // Send shutdown signal
    shutdown_tx.send(true).unwrap();

    // Server should finish within a reasonable time
    let result = tokio::time::timeout(Duration::from_secs(5), server_handle).await;
    assert!(result.is_ok(), "server did not shut down within 5 seconds");
    result.unwrap().unwrap();
}

/// Verify the shutdown timeout mechanism: if a drain future doesn't
/// complete in time, the timeout fires correctly.
#[tokio::test]
async fn shutdown_timeout_fires_when_drain_stalls() {
    let drain = async {
        // Simulate a server that never finishes draining
        tokio::time::sleep(Duration::from_secs(60)).await;
    };

    let result = tokio::time::timeout(Duration::from_millis(100), drain).await;
    assert!(result.is_err(), "timeout should fire for stalled drain");
}

/// Verify watch channel broadcasts to multiple receivers.
#[tokio::test]
async fn watch_channel_broadcasts_shutdown_to_all_receivers() {
    let (tx, _) = watch::channel(false);
    let mut rx1 = tx.subscribe();
    let mut rx2 = tx.subscribe();
    let mut rx3 = tx.subscribe();

    tx.send(true).unwrap();

    // All receivers should see the value
    let r1 = rx1.wait_for(|&v| v).await;
    let r2 = rx2.wait_for(|&v| v).await;
    let r3 = rx3.wait_for(|&v| v).await;
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());
}

/// Verify that remove_stale_socket cleans up existing socket files.
#[tokio::test]
async fn remove_stale_socket_cleans_up() {
    let dir = std::env::temp_dir();
    let path = dir.join("test_stale_socket.sock");
    std::fs::write(&path, b"").unwrap();
    assert!(path.exists());

    tmux_gateway::transports::shutdown::remove_stale_socket(path.to_str().unwrap());
    assert!(!path.exists());
}

/// Verify that remove_stale_socket does not panic on non-existent path.
#[tokio::test]
async fn remove_stale_socket_noop_for_missing() {
    tmux_gateway::transports::shutdown::remove_stale_socket("/tmp/nonexistent_socket_xyz.sock");
    // Should not panic
}
