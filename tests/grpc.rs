mod common;

use tmux_gateway::api::grpc::{
    CapturePaneRequest, KillPaneRequest, KillSessionRequest, KillWindowRequest, ListPanesRequest,
    ListWindowsRequest, LsRequest, NewSessionRequest, NewWindowRequest, RenameSessionRequest,
    SendKeysRequest, SplitWindowRequest, TmuxGateway, TmuxGatewayServiceImpl,
};
use tonic::Request;

#[tokio::test]
async fn ls_returns_ok() {
    if !common::tmux_available() {
        return;
    }
    let service = TmuxGatewayServiceImpl;
    let resp = service.ls(Request::new(LsRequest {})).await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn new_session_returns_name() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    let resp = service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    assert_eq!(resp.into_inner().name, name);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn kill_session_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .kill_session(Request::new(KillSessionRequest { target: name }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn kill_window_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .kill_window(Request::new(KillWindowRequest {
            target: format!("{}:0", name),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn kill_pane_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .kill_pane(Request::new(KillPaneRequest {
            target: format!("{}:0.0", name),
        }))
        .await;
    assert!(resp.is_ok());
}

#[tokio::test]
async fn list_windows_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .list_windows(Request::new(ListWindowsRequest {
            session: name.clone(),
        }))
        .await
        .unwrap();
    assert!(!resp.into_inner().windows.is_empty());
    common::cleanup_session(&name);
}

#[tokio::test]
async fn list_panes_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .list_panes(Request::new(ListPanesRequest {
            target: format!("{}:0", name),
        }))
        .await
        .unwrap();
    assert!(!resp.into_inner().panes.is_empty());
    common::cleanup_session(&name);
}

#[tokio::test]
async fn send_keys_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .send_keys(Request::new(SendKeysRequest {
            target: name.clone(),
            keys: vec!["echo".into(), "Enter".into()],
        }))
        .await;
    assert!(resp.is_ok());
    common::cleanup_session(&name);
}

#[tokio::test]
async fn rename_session_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let new_name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .rename_session(Request::new(RenameSessionRequest {
            target: name,
            new_name: new_name.clone(),
        }))
        .await;
    assert!(resp.is_ok());
    common::cleanup_session(&new_name);
}

#[tokio::test]
async fn new_window_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .new_window(Request::new(NewWindowRequest {
            session: name.clone(),
            name: "testwin".into(),
        }))
        .await
        .unwrap();
    assert_eq!(resp.into_inner().name, "testwin");
    common::cleanup_session(&name);
}

#[tokio::test]
async fn split_window_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .split_window(Request::new(SplitWindowRequest {
            target: format!("{}:0", name),
            horizontal: false,
        }))
        .await;
    assert!(resp.is_ok());
    common::cleanup_session(&name);
}

#[tokio::test]
async fn capture_pane_after_create() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service
        .capture_pane(Request::new(CapturePaneRequest {
            target: format!("{}:0.0", name),
        }))
        .await;
    assert!(resp.is_ok());
    common::cleanup_session(&name);
}

#[tokio::test]
async fn kill_session_nonexistent_returns_error() {
    if !common::tmux_available() {
        return;
    }
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
    if !common::tmux_available() {
        return;
    }
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
    if !common::tmux_available() {
        return;
    }
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
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let service = TmuxGatewayServiceImpl;
    service
        .new_session(Request::new(NewSessionRequest { name: name.clone() }))
        .await
        .unwrap();
    let resp = service.ls(Request::new(LsRequest {})).await.unwrap();
    let sessions = resp.into_inner().sessions;
    assert!(
        sessions.iter().any(|s| s.name == name),
        "session {} not found in gRPC ls",
        name
    );
    common::cleanup_session(&name);
}
