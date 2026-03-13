use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::tmux;

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize, ToSchema)]
struct SessionResponse {
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/sessions",
    responses(
        (status = 200, description = "List tmux sessions", body = Vec<SessionResponse>),
        (status = 500, description = "Failed to list sessions")
    )
)]
async fn list_sessions() -> Result<Json<Vec<SessionResponse>>, (axum::http::StatusCode, String)> {
    let sessions = tmux::list_sessions()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        sessions
            .into_iter()
            .map(|s| SessionResponse {
                name: s.name,
                windows: s.windows,
                created: s.created,
                attached: s.attached,
            })
            .collect(),
    ))
}

#[derive(OpenApi)]
#[openapi(
    paths(health, list_sessions),
    components(schemas(HealthResponse, SessionResponse)),
    info(
        title = "tmux-gateway",
        version = "0.1.0",
        description = "REST API for interacting with local tmux sessions"
    )
)]
pub struct ApiDoc;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/sessions", get(list_sessions))
}
