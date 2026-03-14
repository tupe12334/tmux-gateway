mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tmux_gateway::api::rest;
use tower::ServiceExt;

async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn json_post(uri: &str, json: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

// ── Health ─────────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_200() {
    let app = rest::router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("healthy"));
}

// ── List sessions ──────────────────────────────────────────────────

#[tokio::test]
async fn ls_returns_200() {
    common::require_tmux();

    let app = rest::router();
    let resp = app
        .oneshot(Request::builder().uri("/ls").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let sessions: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(sessions.is_array());
}

// ── Create and kill session ────────────────────────────────────────

#[tokio::test]
async fn create_session_returns_201() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains(&session.name));
}

#[tokio::test]
async fn kill_session_returns_200() {
    let session = common::TestSession::new();

    // Create first
    let app = rest::router();
    let resp = app
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Kill
    let app = rest::router();
    let resp = app
        .oneshot(json_post(
            "/kill-session",
            &format!(r#"{{"target":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── List windows ───────────────────────────────────────────────────

#[tokio::test]
async fn list_windows_returns_200() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(&format!("/list-windows?session={}", session.name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let windows: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(windows.is_array());
    assert!(!windows.as_array().unwrap().is_empty());
}

// ── List panes ─────────────────────────────────────────────────────

#[tokio::test]
async fn list_panes_returns_200() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(&format!("/list-panes?target={}:0", session.name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Send keys ──────────────────────────────────────────────────────

#[tokio::test]
async fn send_keys_returns_200() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(json_post(
            "/send-keys",
            &format!(
                r#"{{"target":"{}:0.0","keys":["echo","Enter"]}}"#,
                session.name
            ),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Rename session ─────────────────────────────────────────────────

#[tokio::test]
async fn rename_session_returns_200() {
    let session = common::TestSession::new();
    let new_session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(json_post(
            "/rename-session",
            &format!(
                r#"{{"target":"{}","new_name":"{}"}}"#,
                session.name, new_session.name
            ),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Rename window ─────────────────────────────────────────────────

#[tokio::test]
async fn rename_window_returns_200() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(json_post(
            "/rename-window",
            &format!(r#"{{"target":"{}:0","new_name":"renamed"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── New window ─────────────────────────────────────────────────────

#[tokio::test]
async fn new_window_returns_201() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(json_post(
            "/new-window",
            &format!(r#"{{"session":"{}","name":"mywin"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

// ── Split window ───────────────────────────────────────────────────

#[tokio::test]
async fn split_window_returns_200() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(json_post(
            "/split-window",
            &format!(r#"{{"target":"{}:0.0","horizontal":false}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Capture pane ───────────────────────────────────────────────────

#[tokio::test]
async fn capture_pane_returns_200() {
    let session = common::TestSession::new();
    let app = rest::router();

    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(&format!("/capture-pane?target={}:0.0", session.name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("content"));
}

// ── Error cases ────────────────────────────────────────────────────

#[tokio::test]
async fn new_session_missing_body_returns_error() {
    let app = rest::router();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/new")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn new_session_invalid_json_returns_error() {
    let app = rest::router();
    let resp = app.oneshot(json_post("/new", "not json")).await.unwrap();

    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn kill_session_nonexistent_returns_error() {
    common::require_tmux();

    let app = rest::router();
    let resp = app
        .oneshot(json_post(
            "/kill-session",
            r#"{"target":"nonexistent_session_xyz_12345"}"#,
        ))
        .await
        .unwrap();

    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[tokio::test]
async fn kill_window_nonexistent_returns_error() {
    common::require_tmux();

    let app = rest::router();
    let resp = app
        .oneshot(json_post("/kill-window", r#"{"target":"nonexistent:0"}"#))
        .await
        .unwrap();

    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[tokio::test]
async fn kill_pane_nonexistent_returns_error() {
    common::require_tmux();

    let app = rest::router();
    let resp = app
        .oneshot(json_post("/kill-pane", r#"{"target":"nonexistent:0.0"}"#))
        .await
        .unwrap();

    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[tokio::test]
async fn kill_session_missing_body_returns_error() {
    let app = rest::router();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/kill-session")
                .header("content-type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn get_on_post_route_returns_405() {
    let app = rest::router();
    let resp = app
        .oneshot(Request::builder().uri("/new").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let app = rest::router();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
