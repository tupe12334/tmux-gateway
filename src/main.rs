use std::env;
use std::time::Duration;
use tmux_gateway::api::middleware;
use tmux_gateway::{cors, export_schemas, port_table, preflight, transports};
use tokio::sync::watch;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    init_tracing();

    let config = preflight::run().await;

    if env::var("EXPORT_SCHEMAS")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false)
    {
        export_schemas::export_all();
    }

    let http_port = config.http_port;
    let grpc_port = config.grpc_port;
    let http_socket = config.http_socket;
    let grpc_socket = config.grpc_socket;

    let shutdown_timeout = env::var("SHUTDOWN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);

    let swagger_url = format!("http://localhost:{}/swagger-ui", http_port);
    let graphql_url = format!("http://localhost:{}/graphql", http_port);
    let grpcui_cmd = format!("grpcui -plaintext localhost:{}", grpc_port);
    let ws_url = format!(
        "https://piehost.com/websocket-tester?url=ws://localhost:{}/ws/pane/{{session}}:{{window}}.{{pane}}?interval_ms=500",
        http_port
    );

    port_table::print_port_table(&[
        ("REST", http_port, swagger_url.as_str()),
        ("GraphQL", http_port, graphql_url.as_str()),
        ("gRPC", grpc_port, grpcui_cmd.as_str()),
        ("WebSocket", http_port, ws_url.as_str()),
    ]);

    if let Some(ref path) = http_socket {
        tracing::info!(path, "HTTP Unix socket enabled");
    }
    if let Some(ref path) = grpc_socket {
        tracing::info!(path, "gRPC Unix socket enabled");
    }

    // Shutdown signal: sender notifies both servers to begin graceful shutdown.
    let (shutdown_tx, _) = watch::channel(false);

    let cors = cors::build_cors_layer(http_port)?;

    info!(
        http_addr = %format!("0.0.0.0:{http_port}"),
        grpc_addr = %format!("0.0.0.0:{grpc_port}"),
        shutdown_timeout_secs = shutdown_timeout,
        tmux_version = %config.tmux_version,
        "Effective configuration"
    );

    let max_body_bytes = env::var("MAX_REQUEST_BODY_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1_048_576); // 1 MB default

    // ── Rate limiting ──────────────────────────────────────────
    let base_rps: u32 = env::var("RATE_LIMIT_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let read_rps: u32 = env::var("RATE_LIMIT_READ_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(base_rps);
    let write_rps: u32 = env::var("RATE_LIMIT_WRITE_RPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(base_rps);

    let read_rate_limit = middleware::RateLimitState::new(read_rps);
    let write_rate_limit = middleware::RateLimitState::new(write_rps);

    tracing::info!(read_rps, write_rps, "Per-IP rate limiting enabled");

    // ── Spawn transports ─────────────────────────────────────────
    let http_app =
        transports::build_http_app(cors, max_body_bytes, read_rate_limit, write_rate_limit);

    let http_handle =
        transports::tcp::http::spawn(http_app.clone(), http_port, shutdown_tx.subscribe()).await?;

    let http_unix_handle = if let Some(ref socket_path) = http_socket {
        Some(transports::unix::http::spawn(http_app, socket_path, shutdown_tx.subscribe()).await?)
    } else {
        None
    };

    let grpc_handle =
        transports::tcp::grpc::spawn(grpc_port, shutdown_tx.subscribe(), shutdown_tx.subscribe())
            .await?;

    let grpc_unix_handle = if let Some(ref socket_path) = grpc_socket {
        Some(transports::unix::grpc::spawn(socket_path, shutdown_tx.subscribe()).await?)
    } else {
        None
    };

    // Wait for shutdown signal (Ctrl+C or SIGTERM).
    transports::shutdown::shutdown_signal().await;
    tracing::info!("Shutdown signal received, draining in-flight requests...");

    // Notify both servers to begin graceful shutdown.
    let _ = shutdown_tx.send(true);

    // Wait for servers to drain, with a timeout.
    let drain = async {
        let _ = tokio::join!(http_handle, grpc_handle);
        if let Some(h) = http_unix_handle {
            let _ = h.await;
        }
        if let Some(h) = grpc_unix_handle {
            let _ = h.await;
        }
    };
    if tokio::time::timeout(Duration::from_secs(shutdown_timeout), drain)
        .await
        .is_err()
    {
        tracing::warn!("Graceful shutdown timed out after {shutdown_timeout}s, forcing exit");
    } else {
        tracing::info!("All servers shut down gracefully");
    }

    Ok(())
}

/// Initializes the tracing subscriber.
/// Set `RUST_LOG_FORMAT=json` for JSON-formatted logs (recommended for production).
fn init_tracing() {
    let filter = EnvFilter::from_default_env().add_directive("tmux_gateway=info".parse().unwrap());

    let use_json = env::var("RUST_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}
