use std::env;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

pub fn build_cors_layer(http_port: u16) -> anyhow::Result<CorsLayer> {
    let origins_raw = env::var("CORS_ORIGINS")
        .unwrap_or_else(|_| format!("http://localhost:{http_port},http://localhost:3000"));
    let raw_entries: Vec<&str> = origins_raw.split(',').map(|s| s.trim()).collect();
    let total = raw_entries.len();
    let mut origins: Vec<http::HeaderValue> = Vec::with_capacity(total);

    for entry in &raw_entries {
        match entry.parse::<http::HeaderValue>() {
            Ok(val) => origins.push(val),
            Err(e) => {
                tracing::warn!(origin = %entry, error = %e, "Invalid CORS origin, skipping");
            }
        }
    }

    if origins.is_empty() {
        anyhow::bail!(
            "No valid CORS origins after parsing CORS_ORIGINS={origins_raw:?}. \
             All {total} entries failed to parse. \
             Fix the CORS_ORIGINS environment variable or remove it to use defaults.",
        );
    }

    let valid = origins.len();
    let invalid = total - valid;

    info!(
        cors_origins = ?origins.iter().map(|o| o.to_str().unwrap_or("<non-utf8>")).collect::<Vec<_>>(),
        cors_valid = valid,
        cors_invalid = invalid,
        "CORS configuration"
    );

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any))
}
