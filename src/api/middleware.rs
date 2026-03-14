use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
};
use http::{Request, StatusCode};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU32;
use std::sync::Arc;
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

/// Generates a UUID v4 for each incoming request.
#[derive(Clone)]
pub struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        id.parse().ok().map(RequestId::new)
    }
}

type IpRateLimiter = RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>;

/// Shared state for the rate-limiting middleware.
#[derive(Clone)]
pub struct RateLimitState {
    limiter: Arc<IpRateLimiter>,
    limit: u32,
}

impl RateLimitState {
    pub fn new(rps: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(rps).expect("RPS must be > 0"))
            .allow_burst(NonZeroU32::new(rps).expect("burst size must be > 0"));
        Self {
            limiter: Arc::new(RateLimiter::keyed(quota)),
            limit: rps,
        }
    }
}

/// Axum middleware that enforces per-IP rate limiting.
/// Returns 429 Too Many Requests when the limit is exceeded.
pub async fn rate_limit(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<RateLimitState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let ip = addr.ip();
    match state.limiter.check_key(&ip) {
        Ok(_) => {
            let mut response = next.run(request).await;
            let headers = response.headers_mut();
            headers.insert("x-ratelimit-limit", state.limit.into());
            response
        }
        Err(not_until) => {
            let wait = not_until.wait_time_from(DefaultClock::default().now());
            let retry_after = wait.as_secs().saturating_add(1);

            let mut response =
                (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response();
            let headers = response.headers_mut();
            headers.insert("x-ratelimit-limit", state.limit.into());
            headers.insert("x-ratelimit-remaining", 0u32.into());
            headers.insert("retry-after", (retry_after as u32).into());
            response
        }
    }
}
