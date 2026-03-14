use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::tmux::{self, TmuxCommands, TmuxError, tmux_interface::Tmux};

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

    async fn list_windows(&self, session: &str) -> Result<Vec<tmux::TmuxWindow>, TmuxError> {
        tmux::list_windows(session).await
    }

    async fn list_panes(&self, target: &str) -> Result<Vec<tmux::TmuxPane>, TmuxError> {
        tmux::list_panes(target).await
    }

    async fn send_keys(&self, target: &str, keys: &[String]) -> Result<(), TmuxError> {
        tmux::send_keys(target, keys).await
    }

    async fn rename_session(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_session(target, new_name).await
    }

    async fn rename_window(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_window(target, new_name).await
    }

    async fn new_window(&self, session: &str, name: &str) -> Result<String, TmuxError> {
        tmux::new_window(session, name).await
    }

    async fn split_window(&self, target: &str, horizontal: bool) -> Result<(), TmuxError> {
        tmux::split_window(target, horizontal).await
    }

    async fn capture_pane(&self, target: &str) -> Result<String, TmuxError> {
        tmux::capture_pane(target).await
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
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

/// Returns `true` if tmux is reachable and responding.
pub fn check_tmux_available() -> bool {
    Tmux::new()
        .version()
        .output()
        .map(|o| o.into_inner().status.success())
        .unwrap_or(false)
}

#[derive(Serialize, ToSchema)]
struct SessionResponse {
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

#[derive(Serialize, ToSchema)]
struct WindowResponse {
    index: u32,
    name: String,
    panes: u32,
    active: bool,
}

#[derive(Serialize, ToSchema)]
struct PaneResponse {
    id: String,
    width: u32,
    height: u32,
    active: bool,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Healthy — tmux is reachable", body = HealthResponse),
        (status = 503, description = "Degraded — tmux is not available", body = HealthResponse)
    )
)]
async fn health() -> (axum::http::StatusCode, Json<HealthResponse>) {
    if check_tmux_available() {
        (
            axum::http::StatusCode::OK,
            Json(HealthResponse {
                status: "healthy".to_string(),
                detail: None,
            }),
        )
    } else {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "degraded".to_string(),
                detail: Some("tmux is not available".to_string()),
            }),
        )
    }
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
                created: DateTime::<Utc>::from_timestamp(s.created, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| s.created.to_string()),
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

#[derive(Deserialize, ToSchema)]
struct ListWindowsRequest {
    session: String,
}

#[utoipa::path(
    get,
    path = "/list-windows",
    params(("session" = String, Query, description = "Session name")),
    responses(
        (status = 200, description = "List windows in session", body = Vec<WindowResponse>),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Failed to list windows")
    )
)]
async fn list_windows(
    axum::extract::Query(params): axum::extract::Query<ListWindowsRequest>,
) -> Result<Json<Vec<WindowResponse>>, (axum::http::StatusCode, String)> {
    let windows = RestHandler
        .list_windows(&params.session)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(Json(
        windows
            .into_iter()
            .map(|w| WindowResponse {
                index: w.index,
                name: w.name,
                panes: w.panes,
                active: w.active,
            })
            .collect(),
    ))
}

#[derive(Deserialize, ToSchema)]
struct ListPanesRequest {
    target: String,
}

#[utoipa::path(
    get,
    path = "/list-panes",
    params(("target" = String, Query, description = "Window target")),
    responses(
        (status = 200, description = "List panes in window", body = Vec<PaneResponse>),
        (status = 404, description = "Window not found"),
        (status = 500, description = "Failed to list panes")
    )
)]
async fn list_panes(
    axum::extract::Query(params): axum::extract::Query<ListPanesRequest>,
) -> Result<Json<Vec<PaneResponse>>, (axum::http::StatusCode, String)> {
    let panes = RestHandler
        .list_panes(&params.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(Json(
        panes
            .into_iter()
            .map(|p| PaneResponse {
                id: p.id,
                width: p.width,
                height: p.height,
                active: p.active,
            })
            .collect(),
    ))
}

#[derive(Deserialize, ToSchema)]
struct SendKeysRequest {
    target: String,
    keys: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/send-keys",
    request_body = SendKeysRequest,
    responses(
        (status = 200, description = "Keys sent"),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to send keys")
    )
)]
async fn send_keys(
    Json(body): Json<SendKeysRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .send_keys(&body.target, &body.keys)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(axum::http::StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
struct RenameRequest {
    target: String,
    new_name: String,
}

#[utoipa::path(
    post,
    path = "/rename-session",
    request_body = RenameRequest,
    responses(
        (status = 200, description = "Session renamed"),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Failed to rename session")
    )
)]
async fn rename_session(
    Json(body): Json<RenameRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .rename_session(&body.target, &body.new_name)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(axum::http::StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/rename-window",
    request_body = RenameRequest,
    responses(
        (status = 200, description = "Window renamed"),
        (status = 404, description = "Window not found"),
        (status = 500, description = "Failed to rename window")
    )
)]
async fn rename_window(
    Json(body): Json<RenameRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .rename_window(&body.target, &body.new_name)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(axum::http::StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
struct NewWindowRequest {
    session: String,
    name: String,
}

#[derive(Serialize, ToSchema)]
struct NewWindowResponse {
    name: String,
}

#[utoipa::path(
    post,
    path = "/new-window",
    request_body = NewWindowRequest,
    responses(
        (status = 201, description = "Window created", body = NewWindowResponse),
        (status = 404, description = "Session not found"),
        (status = 500, description = "Failed to create window")
    )
)]
async fn new_window(
    Json(body): Json<NewWindowRequest>,
) -> Result<(axum::http::StatusCode, Json<NewWindowResponse>), (axum::http::StatusCode, String)> {
    let name = RestHandler
        .new_window(&body.session, &body.name)
        .await
        .map_err(tmux_err_to_http)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(NewWindowResponse { name }),
    ))
}

#[derive(Deserialize, ToSchema)]
struct SplitWindowRequest {
    target: String,
    horizontal: bool,
}

#[utoipa::path(
    post,
    path = "/split-window",
    request_body = SplitWindowRequest,
    responses(
        (status = 200, description = "Window split"),
        (status = 404, description = "Target not found"),
        (status = 500, description = "Failed to split window")
    )
)]
async fn split_window(
    Json(body): Json<SplitWindowRequest>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    RestHandler
        .split_window(&body.target, body.horizontal)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(axum::http::StatusCode::OK)
}

#[derive(Serialize, ToSchema)]
struct CapturePaneResponse {
    content: String,
}

#[utoipa::path(
    get,
    path = "/capture-pane",
    params(("target" = String, Query, description = "Pane target")),
    responses(
        (status = 200, description = "Pane content captured", body = CapturePaneResponse),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to capture pane")
    )
)]
async fn capture_pane(
    axum::extract::Query(params): axum::extract::Query<KillTargetRequest>,
) -> Result<Json<CapturePaneResponse>, (axum::http::StatusCode, String)> {
    let content = RestHandler
        .capture_pane(&params.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(Json(CapturePaneResponse { content }))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        ls,
        new,
        kill_session,
        kill_window,
        kill_pane,
        list_windows,
        list_panes,
        send_keys,
        rename_session,
        rename_window,
        new_window,
        split_window,
        capture_pane
    ),
    components(schemas(
        HealthResponse,
        SessionResponse,
        WindowResponse,
        PaneResponse,
        NewSessionRequest,
        NewSessionResponse,
        KillTargetRequest,
        ListWindowsRequest,
        ListPanesRequest,
        SendKeysRequest,
        RenameRequest,
        NewWindowRequest,
        NewWindowResponse,
        SplitWindowRequest,
        CapturePaneResponse
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
        .route("/list-windows", get(list_windows))
        .route("/list-panes", get(list_panes))
        .route("/send-keys", post(send_keys))
        .route("/rename-session", post(rename_session))
        .route("/rename-window", post(rename_window))
        .route("/new-window", post(new_window))
        .route("/split-window", post(split_window))
        .route("/capture-pane", get(capture_pane))
}
