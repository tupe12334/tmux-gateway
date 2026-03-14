mod common;

use tmux_gateway::api::grpc::{
    CapturePaneRequest, KillPaneRequest, KillSessionRequest, KillWindowRequest, ListPanesRequest,
    ListWindowsRequest, LsRequest, NewSessionRequest, NewWindowRequest, RenameSessionRequest,
    RenameWindowRequest, SendKeysRequest, SplitWindowRequest, TmuxGateway, TmuxGatewayServiceImpl,
};
use tonic::Request;

#[tokio::test]
async fn ls_returns_ok() {
    common::require_tmux();
    let service = TmuxGatewayServiceImpl;
    let resp = service.ls(Request::new(LsRequest {})).await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn new_session_returns_name() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    let resp = service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    assert_eq!(resp.into_inner().name, session.name);
}

#[tokio::test]
async fn kill_session_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .kill_session(Request::new(KillSessionRequest {
            target: session.name.clone(),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn kill_window_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .kill_window(Request::new(KillWindowRequest {
            target: format!("{}:0", session.name),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn kill_pane_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .kill_pane(Request::new(KillPaneRequest {
            target: format!("{}:0.0", session.name),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn list_windows_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .list_windows(Request::new(ListWindowsRequest {
            session: session.name.clone(),
        }))
        .await
        .unwrap();
    assert!(!resp.into_inner().windows.is_empty());
}

#[tokio::test]
async fn list_panes_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .list_panes(Request::new(ListPanesRequest {
            target: format!("{}:0", session.name),
        }))
        .await
        .unwrap();
    assert!(!resp.into_inner().panes.is_empty());
}

#[tokio::test]
async fn send_keys_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .send_keys(Request::new(SendKeysRequest {
            target: format!("{}:0.0", session.name),
            keys: vec!["echo".into(), "Enter".into()],
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn rename_session_after_create() {
    let session = common::TestSession::new();
    let new_session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .rename_session(Request::new(RenameSessionRequest {
            target: session.name.clone(),
            new_name: new_session.name.clone(),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn rename_window_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .rename_window(Request::new(RenameWindowRequest {
            target: format!("{}:0", session.name),
            new_name: "renamed".into(),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn new_window_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .new_window(Request::new(NewWindowRequest {
            session: session.name.clone(),
            name: "testwin".into(),
        }))
        .await
        .unwrap();
    assert_eq!(resp.into_inner().name, "testwin");
}

#[tokio::test]
async fn split_window_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .split_window(Request::new(SplitWindowRequest {
            target: format!("{}:0.0", session.name),
            horizontal: false,
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn capture_pane_after_create() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service
        .capture_pane(Request::new(CapturePaneRequest {
            target: format!("{}:0.0", session.name),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn kill_session_nonexistent_returns_error() {
    common::require_tmux();
    let service = TmuxGatewayServiceImpl;
    let resp = service
        .kill_session(Request::new(KillSessionRequest {
            target: "nonexistent_grpc_xyz_99999".into(),
        }))
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn kill_window_nonexistent_returns_error() {
    common::require_tmux();
    let service = TmuxGatewayServiceImpl;
    let resp = service
        .kill_window(Request::new(KillWindowRequest {
            target: "nonexistent:0".into(),
        }))
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn kill_pane_nonexistent_returns_error() {
    common::require_tmux();
    let service = TmuxGatewayServiceImpl;
    let resp = service
        .kill_pane(Request::new(KillPaneRequest {
            target: "nonexistent:0.0".into(),
        }))
        .await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn ls_includes_created_session() {
    let session = common::TestSession::new();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest {
            name: session.name.clone(),
        }))
        .await
        .unwrap();
    let resp = service.ls(Request::new(LsRequest {})).await.unwrap();
    let sessions = resp.into_inner().sessions;
    assert!(
        sessions.iter().any(|s| s.name == session.name),
        "session {} not found in gRPC ls",
        session.name
    );
}
