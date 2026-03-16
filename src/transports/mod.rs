pub mod shutdown;
pub mod tcp;
pub mod unix;

use crate::api::{graphql, middleware, rest, ws};
use axum::extract::DefaultBodyLimit;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing::Span;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn build_http_app(
    cors: CorsLayer,
    max_body_bytes: usize,
    read_rate_limit: middleware::RateLimitState,
    write_rate_limit: middleware::RateLimitState,
) -> axum::Router {
    let x_request_id = http::HeaderName::from_static("x-request-id");

    axum::Router::new()
        .merge(ws::router())
        .merge(
            rest::read_router().route_layer(axum::middleware::from_fn_with_state(
                read_rate_limit,
                middleware::rate_limit,
            )),
        )
        .merge(
            rest::write_router().route_layer(axum::middleware::from_fn_with_state(
                write_rate_limit.clone(),
                middleware::rate_limit,
            )),
        )
        .merge(
            graphql::router().route_layer(axum::middleware::from_fn_with_state(
                write_rate_limit,
                middleware::rate_limit,
            )),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", rest::ApiDoc::openapi()))
        .layer(DefaultBodyLimit::max(max_body_bytes))
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
        ))
}
