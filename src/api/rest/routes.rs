use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::tmux::{self, RealTmuxExecutor, TmuxCommands, TmuxError};

struct RestHandler;

impl TmuxCommands for RestHandler {}

fn tmux_err_to_http(e: TmuxError) -> (StatusCode, String) {
    let status = match &e {
        TmuxError::SessionNotFound(_)
        | TmuxError::WindowNotFound(_)
        | TmuxError::PaneNotFound(_) => StatusCode::NOT_FOUND,
        TmuxError::SessionAlreadyExists(_) => StatusCode::CONFLICT,
        TmuxError::InvalidTarget(_) | TmuxError::Validation(_) | TmuxError::ParseError { .. } => {
            StatusCode::BAD_REQUEST
        }
        TmuxError::TmuxNotRunning | TmuxError::CommandFailed { .. } => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (status, e.to_string())
}

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct SessionResponse {
    id: String,
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

#[derive(Serialize, ToSchema)]
struct WindowResponse {
    id: String,
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
    current_path: String,
    current_command: String,
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
    if tmux::is_available(&RealTmuxExecutor).await {
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
                id: s.id,
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
    id: String,
    name: String,
    windows: u32,
    created: String,
    attached: bool,
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
    let session = RestHandler
        .create_session(&body.name)
        .await
        .map_err(tmux_err_to_http)?;

    Ok((
        StatusCode::CREATED,
        Json(NewSessionResponse {
            id: session.id,
            name: session.name,
            windows: session.windows,
            created: DateTime::<Utc>::from_timestamp(session.created, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| session.created.to_string()),
            attached: session.attached,
        }),
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
                id: w.id,
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
                current_path: p.current_path,
                current_command: p.current_command,
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
    id: String,
    index: u32,
    name: String,
    panes: u32,
    active: bool,
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
    let window = RestHandler
        .new_window(&body.session, &body.name)
        .await
        .map_err(tmux_err_to_http)?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(NewWindowResponse {
            id: window.id,
            index: window.index,
            name: window.name,
            panes: window.panes,
            active: window.active,
        }),
    ))
}

#[derive(Deserialize, ToSchema)]
struct SplitWindowRequest {
    target: String,
    horizontal: bool,
}

#[derive(Serialize, ToSchema)]
struct SplitWindowResponse {
    id: String,
    width: u32,
    height: u32,
    active: bool,
    current_path: String,
    current_command: String,
}

#[utoipa::path(
    post,
    path = "/split-window",
    request_body = SplitWindowRequest,
    responses(
        (status = 200, description = "Window split", body = SplitWindowResponse),
        (status = 404, description = "Target not found"),
        (status = 500, description = "Failed to split window")
    )
)]
async fn split_window(
    Json(body): Json<SplitWindowRequest>,
) -> Result<Json<SplitWindowResponse>, (axum::http::StatusCode, String)> {
    let pane = RestHandler
        .split_window(&body.target, body.horizontal)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(Json(SplitWindowResponse {
        id: pane.id,
        width: pane.width,
        height: pane.height,
        active: pane.active,
        current_path: pane.current_path,
        current_command: pane.current_command,
    }))
}

#[derive(Deserialize, ToSchema)]
struct CreateSessionWithWindowsRequest {
    name: String,
    window_names: Vec<String>,
}

#[derive(Serialize, ToSchema)]
struct CreateSessionWithWindowsResponse {
    id: String,
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

#[utoipa::path(
    post,
    path = "/create-session-with-windows",
    request_body = CreateSessionWithWindowsRequest,
    responses(
        (status = 201, description = "Session created with windows", body = CreateSessionWithWindowsResponse),
        (status = 400, description = "Invalid session or window name"),
        (status = 409, description = "Session already exists"),
        (status = 500, description = "Failed to create session")
    )
)]
async fn create_session_with_windows(
    Json(body): Json<CreateSessionWithWindowsRequest>,
) -> Result<(StatusCode, Json<CreateSessionWithWindowsResponse>), (StatusCode, String)> {
    let session = RestHandler
        .create_session_with_windows(&body.name, &body.window_names)
        .await
        .map_err(tmux_err_to_http)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionWithWindowsResponse {
            id: session.id,
            name: session.name,
            windows: session.windows,
            created: DateTime::<Utc>::from_timestamp(session.created, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| session.created.to_string()),
            attached: session.attached,
        }),
    ))
}

#[derive(Deserialize, ToSchema)]
struct SwapPanesRequest {
    src: String,
    dst: String,
}

#[utoipa::path(
    post,
    path = "/swap-panes",
    request_body = SwapPanesRequest,
    responses(
        (status = 200, description = "Panes swapped"),
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to swap panes")
    )
)]
async fn swap_panes(
    Json(body): Json<SwapPanesRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .swap_panes(&body.src, &body.dst)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
struct MoveWindowRequest {
    source: String,
    destination_session: String,
}

#[utoipa::path(
    post,
    path = "/move-window",
    request_body = MoveWindowRequest,
    responses(
        (status = 200, description = "Window moved"),
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Session or window not found"),
        (status = 500, description = "Failed to move window")
    )
)]
async fn move_window(
    Json(body): Json<MoveWindowRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .move_window(&body.source, &body.destination_session)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/select-window",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Window selected"),
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Window not found"),
        (status = 500, description = "Failed to select window")
    )
)]
async fn select_window(
    Json(body): Json<KillTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .select_window(&body.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/select-pane",
    request_body = KillTargetRequest,
    responses(
        (status = 200, description = "Pane selected"),
        (status = 400, description = "Invalid target"),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to select pane")
    )
)]
async fn select_pane(
    Json(body): Json<KillTargetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    RestHandler
        .select_pane(&body.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
struct ResizePaneRequest {
    target: String,
    direction: String,
    amount: u32,
}

#[utoipa::path(
    post,
    path = "/resize-pane",
    request_body = ResizePaneRequest,
    responses(
        (status = 200, description = "Pane resized"),
        (status = 400, description = "Invalid target or direction"),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to resize pane")
    )
)]
async fn resize_pane(
    Json(body): Json<ResizePaneRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let direction = match body.direction.as_str() {
        "up" | "Up" | "U" => tmux::ResizeDirection::Up(body.amount),
        "down" | "Down" | "D" => tmux::ResizeDirection::Down(body.amount),
        "left" | "Left" | "L" => tmux::ResizeDirection::Left(body.amount),
        "right" | "Right" | "R" => tmux::ResizeDirection::Right(body.amount),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("invalid direction: {}", body.direction),
            ));
        }
    };
    RestHandler
        .resize_pane(&body.target, direction)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
struct SelectLayoutRequest {
    target: String,
    layout: String,
}

#[utoipa::path(
    post,
    path = "/select-layout",
    request_body = SelectLayoutRequest,
    responses(
        (status = 200, description = "Layout applied"),
        (status = 400, description = "Invalid target or layout"),
        (status = 404, description = "Window not found"),
        (status = 500, description = "Failed to select layout")
    )
)]
async fn select_layout(
    Json(body): Json<SelectLayoutRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let layout = parse_layout(&body.layout).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    RestHandler
        .select_layout(&body.target, layout)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(StatusCode::OK)
}

fn parse_layout(s: &str) -> Result<tmux::PaneLayout, String> {
    match s {
        "even-horizontal" => Ok(tmux::PaneLayout::EvenHorizontal),
        "even-vertical" => Ok(tmux::PaneLayout::EvenVertical),
        "main-horizontal" => Ok(tmux::PaneLayout::MainHorizontal),
        "main-vertical" => Ok(tmux::PaneLayout::MainVertical),
        "tiled" => Ok(tmux::PaneLayout::Tiled),
        other => {
            if other.is_empty() {
                Err("layout must not be empty".to_string())
            } else {
                Ok(tmux::PaneLayout::Custom(other.to_string()))
            }
        }
    }
}

#[derive(Deserialize, ToSchema)]
struct CapturePaneRequest {
    target: String,
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
    axum::extract::Query(params): axum::extract::Query<CapturePaneRequest>,
) -> Result<Json<CapturePaneResponse>, (axum::http::StatusCode, String)> {
    let content = RestHandler
        .capture_pane(&params.target)
        .await
        .map_err(tmux_err_to_http)?;

    Ok(Json(CapturePaneResponse { content }))
}

#[derive(Deserialize, ToSchema)]
struct CapturePaneWithOptionsRequest {
    target: String,
    #[serde(default)]
    start_line: Option<i32>,
    #[serde(default)]
    end_line: Option<i32>,
    #[serde(default)]
    escape_sequences: bool,
}

#[utoipa::path(
    post,
    path = "/capture-pane-with-options",
    request_body = CapturePaneWithOptionsRequest,
    responses(
        (status = 200, description = "Pane content captured with options", body = CapturePaneResponse),
        (status = 404, description = "Pane not found"),
        (status = 500, description = "Failed to capture pane")
    )
)]
async fn capture_pane_with_options(
    Json(body): Json<CapturePaneWithOptionsRequest>,
) -> Result<Json<CapturePaneResponse>, (axum::http::StatusCode, String)> {
    let opts = tmux::CaptureOptions {
        start_line: body.start_line,
        end_line: body.end_line,
        escape_sequences: body.escape_sequences,
    };
    let content = RestHandler
        .capture_pane_with_options(&body.target, &opts)
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
        capture_pane,
        capture_pane_with_options,
        create_session_with_windows,
        swap_panes,
        move_window,
        select_window,
        select_pane,
        resize_pane,
        select_layout
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
        SplitWindowResponse,
        CapturePaneRequest,
        CapturePaneResponse,
        CapturePaneWithOptionsRequest,
        CreateSessionWithWindowsRequest,
        CreateSessionWithWindowsResponse,
        SwapPanesRequest,
        MoveWindowRequest,
        ResizePaneRequest,
        SelectLayoutRequest
    )),
    info(
        title = "tmux-gateway",
        version = "0.1.0",
        description = "REST API for interacting with local tmux sessions"
    )
)]
pub struct ApiDoc;

/// Read-only (GET) routes — typically given a higher rate limit.
pub fn read_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ls", get(ls))
        .route("/list-windows", get(list_windows))
        .route("/list-panes", get(list_panes))
        .route("/capture-pane", get(capture_pane))
}

/// Mutating (POST) routes — typically given a lower rate limit.
pub fn write_router() -> Router {
    Router::new()
        .route("/new", post(new))
        .route("/kill-session", post(kill_session))
        .route("/kill-window", post(kill_window))
        .route("/kill-pane", post(kill_pane))
        .route("/send-keys", post(send_keys))
        .route("/rename-session", post(rename_session))
        .route("/rename-window", post(rename_window))
        .route("/new-window", post(new_window))
        .route("/split-window", post(split_window))
        .route(
            "/create-session-with-windows",
            post(create_session_with_windows),
        )
        .route(
            "/capture-pane-with-options",
            post(capture_pane_with_options),
        )
        .route("/swap-panes", post(swap_panes))
        .route("/move-window", post(move_window))
        .route("/select-window", post(select_window))
        .route("/select-pane", post(select_pane))
        .route("/resize-pane", post(resize_pane))
        .route("/select-layout", post(select_layout))
}

pub fn router() -> Router {
    read_router().merge(write_router())
}
