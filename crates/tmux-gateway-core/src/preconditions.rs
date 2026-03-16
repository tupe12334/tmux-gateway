use crate::TmuxError;
use crate::executor::TmuxExecutor;
use crate::has_session::has_session;
use crate::list_panes::list_panes;
use crate::list_windows::list_windows;
use crate::validation::{PaneTarget, SessionName, WindowTarget};

/// Ensure a session exists before a mutating operation.
///
/// Uses the efficient `has-session` tmux command (O(1) exit-code check).
pub async fn require_session_exists(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
) -> Result<(), TmuxError> {
    if !has_session(executor, name).await? {
        return Err(TmuxError::SessionNotFound(name.as_str().to_string()));
    }
    Ok(())
}

/// Ensure a session does NOT exist before creation (duplicate prevention).
pub async fn require_session_not_exists(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
) -> Result<(), TmuxError> {
    if has_session(executor, name).await? {
        return Err(TmuxError::SessionAlreadyExists(name.as_str().to_string()));
    }
    Ok(())
}

/// Ensure the window referenced by a `WindowTarget` exists.
///
/// Verifies that the session exists and contains the named/indexed window.
pub async fn require_window_exists(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
) -> Result<(), TmuxError> {
    let session = SessionName::try_from(target.session())?;
    let windows = list_windows(executor, &session).await?;
    let window_part = target.window();

    let found = windows
        .iter()
        .any(|w| w.name == window_part || w.index.to_string() == window_part);

    if !found {
        return Err(TmuxError::WindowNotFound(target.as_str().to_string()));
    }
    Ok(())
}

/// Ensure the pane referenced by a `PaneTarget` exists.
///
/// Verifies that the session, window, and pane all exist by listing panes
/// for the target window and checking the pane index or ID.
pub async fn require_pane_exists(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
) -> Result<(), TmuxError> {
    let window_target =
        WindowTarget::try_from(format!("{}:{}", target.session(), target.window()))?;
    let panes = list_panes(executor, &window_target).await?;
    let pane_part = target.pane();

    let found = if let Ok(pane_idx) = pane_part.parse::<usize>() {
        pane_idx < panes.len()
    } else {
        panes.iter().any(|p| p.id == pane_part)
    };

    if !found {
        return Err(TmuxError::PaneNotFound(target.as_str().to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockExecutor {
        responses: Vec<TmuxOutput>,
        call_count: AtomicUsize,
    }

    impl MockExecutor {
        fn new(responses: Vec<TmuxOutput>) -> Self {
            Self {
                responses,
                call_count: AtomicUsize::new(0),
            }
        }
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.responses[idx].clone())
        }
    }

    // ── require_session_exists ──

    #[tokio::test]
    async fn session_exists_passes() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: String::new(),
            success: true,
        }]);
        let name = SessionName::try_from("my-session").unwrap();
        assert!(require_session_exists(&executor, &name).await.is_ok());
    }

    #[tokio::test]
    async fn session_exists_fails_when_missing() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: "can't find session: nosession".to_string(),
            success: false,
        }]);
        let name = SessionName::try_from("nosession").unwrap();
        let err = require_session_exists(&executor, &name).await.unwrap_err();
        assert!(matches!(err, TmuxError::SessionNotFound(ref n) if n == "nosession"));
    }

    #[tokio::test]
    async fn session_exists_propagates_tmux_not_running() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: "no server running on /tmp/tmux-1000/default".to_string(),
            success: false,
        }]);
        let name = SessionName::try_from("test").unwrap();
        let err = require_session_exists(&executor, &name).await.unwrap_err();
        assert!(matches!(err, TmuxError::TmuxNotRunning));
    }

    // ── require_session_not_exists ──

    #[tokio::test]
    async fn session_not_exists_passes_when_missing() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: "can't find session: new-session".to_string(),
            success: false,
        }]);
        let name = SessionName::try_from("new-session").unwrap();
        assert!(require_session_not_exists(&executor, &name).await.is_ok());
    }

    #[tokio::test]
    async fn session_not_exists_fails_when_present() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: String::new(),
            success: true,
        }]);
        let name = SessionName::try_from("existing").unwrap();
        let err = require_session_not_exists(&executor, &name)
            .await
            .unwrap_err();
        assert!(matches!(err, TmuxError::SessionAlreadyExists(ref n) if n == "existing"));
    }

    // ── require_window_exists ──

    #[tokio::test]
    async fn window_exists_passes_by_name() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: "@0\t0\tbash\t1\t1\n@1\t1\tvim\t1\t0\n".to_string(),
            stderr: String::new(),
            success: true,
        }]);
        let target = WindowTarget::try_from("sess:vim").unwrap();
        assert!(require_window_exists(&executor, &target).await.is_ok());
    }

    #[tokio::test]
    async fn window_exists_passes_by_index() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: "@0\t0\tbash\t1\t1\n@1\t1\tvim\t1\t0\n".to_string(),
            stderr: String::new(),
            success: true,
        }]);
        let target = WindowTarget::try_from("sess:1").unwrap();
        assert!(require_window_exists(&executor, &target).await.is_ok());
    }

    #[tokio::test]
    async fn window_exists_fails_when_missing() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: "@0\t0\tbash\t1\t1\n".to_string(),
            stderr: String::new(),
            success: true,
        }]);
        let target = WindowTarget::try_from("sess:vim").unwrap();
        let err = require_window_exists(&executor, &target).await.unwrap_err();
        assert!(matches!(err, TmuxError::WindowNotFound(ref t) if t == "sess:vim"));
    }

    #[tokio::test]
    async fn window_exists_fails_when_session_missing() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: "session not found: nosess".to_string(),
            success: false,
        }]);
        let target = WindowTarget::try_from("nosess:0").unwrap();
        let err = require_window_exists(&executor, &target).await.unwrap_err();
        assert!(matches!(err, TmuxError::SessionNotFound(_)));
    }

    // ── require_pane_exists ──

    #[tokio::test]
    async fn pane_exists_passes() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: "%0\t80\t24\t1\t/home\tbash\t1234\n%1\t80\t24\t0\t/tmp\tzsh\t5678\n"
                .to_string(),
            stderr: String::new(),
            success: true,
        }]);
        let target = PaneTarget::try_from("sess:win.1").unwrap();
        assert!(require_pane_exists(&executor, &target).await.is_ok());
    }

    #[tokio::test]
    async fn pane_exists_fails_index_out_of_range() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: "%0\t80\t24\t1\t/home\tbash\t1234\n".to_string(),
            stderr: String::new(),
            success: true,
        }]);
        let target = PaneTarget::try_from("sess:win.5").unwrap();
        let err = require_pane_exists(&executor, &target).await.unwrap_err();
        assert!(matches!(err, TmuxError::PaneNotFound(ref t) if t == "sess:win.5"));
    }

    #[tokio::test]
    async fn pane_exists_fails_when_window_missing() {
        let executor = MockExecutor::new(vec![TmuxOutput {
            stdout: String::new(),
            stderr: "window not found: nowin".to_string(),
            success: false,
        }]);
        let target = PaneTarget::try_from("sess:nowin.0").unwrap();
        let err = require_pane_exists(&executor, &target).await.unwrap_err();
        assert!(matches!(err, TmuxError::WindowNotFound(_)));
    }
}
