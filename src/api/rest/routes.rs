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

    async fn new(&self, name: &str) -> Result<String, String> {
        tmux::new_session(name).await
    }

    async fn kill_session(&self, target: &str) -> Result<(), String> {
        tmux::kill_session(target).await
    }

    async fn kill_window(&self, target: &str) -> Result<(), String> {
        tmux::kill_window(target).await
    }

    async fn kill_pane(&self, target: &str) -> Result<(), String> {
        tmux::kill_pane(target).await
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
        .new(&body.name)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(NewSessionResponse { name }),
    ))
}

#[derive(Deserialize, ToSchema)]
struct KillTargetRequest {
    target: String,
}

#[utoipa::path(
    post,
    path = "/kill-session",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Session killed"),
        (status = 500, description = "Failed to kill session")
    )
)]
async fn kill_session(
    Json(body): Json<KillTargetRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .kill_session(&body.target)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(axum::http::StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/kill-window",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Window killed"),
        (status = 500, description = "Failed to kill window")
    )
)]
async fn kill_window(
    Json(body): Json<KillTargetRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .kill_window(&body.target)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(axum::http::StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/kill-pane",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Pane killed"),
        (status = 500, description = "Failed to kill pane")
    )
)]
async fn kill_pane(
    Json(body): Json<KillTargetRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .kill_pane(&body.target)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(axum::http::StatusCode::OK)
}

#[derive(OpenApi)]
#[openapi(
    paths(health, ls, new, kill_session, kill_window, kill_pane),
    components(schemas(HealthResponse, SessionResponse, NewSessionRequest, NewSessionResponse, KillTargetRequest)),
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
        .route("/kill-session", post(kill_session))
        .route("/kill-window", post(kill_window))
        .route("/kill-pane", post(kill_pane))
}
