use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::tmux::{self, TmuxCommands};

struct RestHandler;

impl TmuxCommands for RestHandler {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, String> {
        tmux::list_sessions().await
    }

    async fn new_session(&self, name: &str) -> Result<String, String> {
        tmux::new_session(name).await
    }
}

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
    path = "/ls",
    responses(
        (status = 200, description = "List tmux sessions", body = Vec<SessionResponse>),
        (status = 500, description = "Failed to list sessions")
    )
)]
async fn ls() -> Result<Json<Vec<SessionResponse>>, (axum::http::StatusCode, String)> {
    let sessions = RestHandler
        .ls()
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

#[derive(Deserialize, ToSchema)]
struct NewSessionRequest {
    name: String,
}

#[derive(Serialize, ToSchema)]
struct NewSessionResponse {
    name: String,
}

#[utoipa::path(
    post,
    path = "/new",
    request_body = NewSessionRequest,
    responses(
        (status = 201, description = "Session created", body = NewSessionResponse),
        (status = 500, description = "Failed to create session")
    )
)]
async fn new(
    Json(body): Json<NewSessionRequest>,
) -> Result<(axum::http::StatusCode, Json<NewSessionResponse>), (axum::http::StatusCode, String)> {
    let name = RestHandler
        .new_session(&body.name)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(NewSessionResponse { name }),
    ))
}

#[derive(OpenApi)]
#[openapi(
    paths(health, ls, new),
    components(schemas(HealthResponse, SessionResponse, NewSessionRequest, NewSessionResponse)),
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
        .route("/ls", get(ls))
        .route("/new", post(new))
}
