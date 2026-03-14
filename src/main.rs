use std::env;
use std::time::Duration;
use tmux_gateway::api::{graphql, grpc, middleware, rest};
use tmux_gateway::{export_schemas, port_table, preflight};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::watch;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Span};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    init_tracing();

    let config = preflight::run();

    export_schemas::export_all();

    let http_port = config.http_port;
    let grpc_port = config.grpc_port;

    let shutdown_timeout = env::var("SHUTDOWN_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);

    let swagger_url = format!("http://localhost:{}/swagger-ui", http_port);
    let graphql_url = format!("http://localhost:{}/graphql", http_port);
    let grpcui_cmd = format!("grpcui -plaintext localhost:{}", grpc_port);

    port_table::print_port_table(&[
        ("REST", http_port, swagger_url.as_str()),
        ("GraphQL", http_port, graphql_url.as_str()),
        ("gRPC", grpc_port, grpcui_cmd.as_str()),
    ]);

    // Shutdown signal: sender notifies both servers to begin graceful shutdown.
    let (shutdown_tx, _) = watch::channel(false);
    let mut http_shutdown_rx = shutdown_tx.subscribe();
    let mut grpc_shutdown_rx = shutdown_tx.subscribe();

    let cors = {
        let origins_raw = env::var("CORS_ORIGINS").unwrap_or_else(|_| {
            format!(
                "http://localhost:{},http://localhost:{}",
                http_port, grpc_port
            )
        });
        let raw_entries: Vec<&str> = origins_raw.split(',').map(|s| s.trim()).collect();
        let total = raw_entries.len();
        let origins: Vec<http::HeaderValue> = raw_entries
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        let valid = origins.len();
        let invalid = total - valid;

        info!(
            http_addr = %format!("0.0.0.0:{http_port}"),
            grpc_addr = %format!("0.0.0.0:{grpc_port}"),
            cors_origins = ?origins.iter().map(|o| o.to_str().unwrap_or("<non-utf8>")).collect::<Vec<_>>(),
            cors_valid = valid,
            cors_invalid = invalid,
            shutdown_timeout_secs = shutdown_timeout,
            tmux_version = %config.tmux_version,
            "Effective configuration"
        );

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    };

    let x_request_id = http::HeaderName::from_static("x-request-id");

    let http_app = axum::Router::new()
        .merge(rest::router())
        .merge(graphql::router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", rest::ApiDoc::openapi()))
        .layer(cors)
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &http::Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("-");
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        path = %request.uri().path(),
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
        .layer(SetRequestIdLayer::new(
            x_request_id,
            middleware::UuidRequestId,
        ));

    let http_addr = format!("0.0.0.0:{http_port}");
    let http_handle = tokio::spawn(async move {
        let listener = TcpListener::bind(&http_addr).await.unwrap();
        tracing::info!("HTTP server (REST + GraphQL + Swagger) listening on {http_addr}");
        axum::serve(listener, http_app)
            .with_graceful_shutdown(async move {
                let _ = http_shutdown_rx.wait_for(|&v| v).await;
                tracing::info!("HTTP server shutting down...");
            })
            .await
            .unwrap();
    });

    let grpc_addr = format!("0.0.0.0:{grpc_port}");
    let grpc_handle = tokio::spawn(async move {
        let addr = grpc_addr.parse().unwrap();
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_file_descriptor_set(grpc::file_descriptor_set())
            .build_v1()
            .unwrap();
        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        // Check tmux availability for gRPC health status
        if rest::check_tmux_available() {
            health_reporter
                .set_serving::<grpc::TmuxGatewayServerConcrete>()
                .await;
        } else {
            health_reporter
                .set_not_serving::<grpc::TmuxGatewayServerConcrete>()
                .await;
        }

        tracing::info!("gRPC server listening on {}", addr);
        tonic::transport::Server::builder()
            .add_service(health_service)
            .add_service(grpc::grpc_server())
            .add_service(reflection_service)
            .serve_with_shutdown(addr, async move {
                let _ = grpc_shutdown_rx.wait_for(|&v| v).await;
                tracing::info!("gRPC server shutting down...");
            })
            .await
            .unwrap();
    });

    // Wait for shutdown signal (Ctrl+C or SIGTERM).
    shutdown_signal().await;
    tracing::info!("Shutdown signal received, draining in-flight requests...");

    // Notify both servers to begin graceful shutdown.
    let _ = shutdown_tx.send(true);

    // Wait for servers to drain, with a timeout.
    let drain = async {
        let _ = tokio::join!(http_handle, grpc_handle);
    };
    if tokio::time::timeout(Duration::from_secs(shutdown_timeout), drain)
        .await
        .is_err()
    {
        tracing::warn!("Graceful shutdown timed out after {shutdown_timeout}s, forcing exit");
    } else {
        tracing::info!("All servers shut down gracefully");
    }
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

async fn shutdown_signal() {
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
