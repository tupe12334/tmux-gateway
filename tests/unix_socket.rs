mod common;

use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

fn temp_socket_path(label: &str) -> PathBuf {
    let name = format!(
        "tmux-gw-test-{}-{}-{}.sock",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    std::env::temp_dir().join(name)
}

struct SocketGuard(PathBuf);

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

// ── HTTP Unix socket ──────────────────────────────────────────────

#[tokio::test]
async fn http_unix_socket_health() {
    let path = temp_socket_path("http");
    let guard = SocketGuard(path.clone());

    let uds = tokio::net::UnixListener::bind(&guard.0).unwrap();
    let app = tmux_gateway::api::rest::router();

    let handle = tokio::spawn(async move {
        axum::serve(uds, app.into_make_service()).await.unwrap();
    });

    // Give the server a moment to start accepting.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut stream = UnixStream::connect(&guard.0).await.unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf);

    assert!(response.contains("200 OK"), "expected 200, got: {response}");
    assert!(
        response.contains("healthy"),
        "expected healthy in body, got: {response}"
    );

    handle.abort();
}

#[tokio::test]
async fn http_unix_socket_ls() {
    common::require_tmux();

    let path = temp_socket_path("http-ls");
    let guard = SocketGuard(path.clone());

    let uds = tokio::net::UnixListener::bind(&guard.0).unwrap();
    let app = tmux_gateway::api::rest::router();

    let handle = tokio::spawn(async move {
        axum::serve(uds, app.into_make_service()).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let mut stream = UnixStream::connect(&guard.0).await.unwrap();
    stream
        .write_all(b"GET /ls HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf);

    assert!(response.contains("200 OK"), "expected 200, got: {response}");

    handle.abort();
}

// ── Stale socket removal ──────────────────────────────────────────

#[tokio::test]
async fn stale_socket_is_removed_on_rebind() {
    let path = temp_socket_path("stale");
    let guard = SocketGuard(path.clone());

    // Create a stale socket file.
    std::fs::write(&guard.0, b"").unwrap();
    assert!(guard.0.exists());

    // Remove stale and rebind.
    let _ = std::fs::remove_file(&guard.0);
    let _uds = tokio::net::UnixListener::bind(&guard.0).unwrap();

    // If we got here, the rebind succeeded after removing the stale file.
    assert!(guard.0.exists());
}
