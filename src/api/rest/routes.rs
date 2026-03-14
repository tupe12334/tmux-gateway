use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::tmux::{self, TmuxCommands, TmuxError};

struct RestHandler;

impl TmuxCommands for RestHandler {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, TmuxError> {
        tmux::list_sessions().await
    }

    async fn create_session(&self, name: &str) -> Result<String, TmuxError> {
        tmux::new_session(name).await
    }

    async fn kill_session(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_session(target).await
    }

    async fn kill_window(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_window(target).await
    }

    async fn kill_pane(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_pane(target).await
    }
}

fn tmux_err_to_http(e: TmuxError) -> (StatusCode, String) {
    let status =
        StatusCode::from_u16(e.http_status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, e.to_string())
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
async fn ls() -> Result<Json<Vec<SessionResponse>>, (StatusCode, String)> {
    let sessions = RestHandler.ls().await.map_err(tmux_err_to_http)?;

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
        (status = 400, description = "Invalid session name"),
        (status = 500, description = "Failed to create session")
    )
)]
async fn new(
    Json(body): Json<NewSessionRequest>,
) -> Result<(StatusCode, Json<NewSessionResponse>), (StatusCode, String)> {
    let name = RestHandler
        .create_session(&body.name)
        .await
        .map_err(tmux_err_to_http)?;

    Ok((StatusCode::CREATED, Json(NewSessionResponse { name })))
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
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Failed to kill session")
    )
)]
async fn kill_session(
    Json(body): Json<KillTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .kill_session(&body.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/kill-window",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Window killed"),
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Window not found"),
        (status = 500, description = "Failed to kill window")
    )
)]
async fn kill_window(
    Json(body): Json<KillTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .kill_window(&body.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/kill-pane",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Pane killed"),
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to kill pane")
    )
)]
async fn kill_pane(
    Json(body): Json<KillTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .kill_pane(&body.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[derive(OpenApi)]
#[openapi(
    paths(health, ls, new, kill_session, kill_window, kill_pane),
    components(schemas(
        HealthResponse,
        SessionResponse,
        NewSessionRequest,
        NewSessionResponse,
        KillTargetRequest
    )),
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
