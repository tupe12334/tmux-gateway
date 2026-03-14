use crate::TmuxWindow;
use crate::executor::TmuxExecutor;
use crate::list_windows::get_window;
use crate::new_window::new_window;
use crate::validation::{validate_session_target, validate_window_name};

use super::TmuxError;

/// Create a window if it doesn't exist in the session, or return the existing one.
///
/// This avoids the TOCTOU race inherent in check-then-create by attempting
/// the create first and falling back to a lookup if tmux reports the window
/// already exists (via `CommandFailed` with a "create window failed" stderr).
///
/// Note: tmux does not return a typed "window already exists" error—it allows
/// duplicate window names. So we first check if a window with the given name
/// exists, and only create if it doesn't.
#[tracing::instrument(skip(executor))]
pub async fn ensure_window(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
    name: &str,
) -> Result<TmuxWindow, TmuxError> {
    validate_session_target(session)?;
    validate_window_name(name)?;

    // tmux allows duplicate window names, so we check first to avoid duplicates.
    if let Some(window) = get_window(executor, session, name).await? {
        return Ok(window);
    }

    // Window doesn't exist yet—create it.
    // If another caller created it between our check and this create,
    // we'll end up with a duplicate name. This is best-effort since
    // tmux doesn't support transactions or unique window name constraints.
    new_window(executor, session, name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    /// Mock where window doesn't exist yet, so create succeeds.
    struct MockNoExistingWindow;

    impl TmuxExecutor for MockNoExistingWindow {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            match args.first() {
                Some(&"list-windows") => Ok(TmuxOutput {
                    stdout: "@0\t0\tbash\t1\t1\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                Some(&"new-window") => Ok(TmuxOutput {
                    stdout: "@1\t1\tmy-win\t1\t0\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                _ => panic!("unexpected command: {args:?}"),
            }
        }
    }

    #[tokio::test]
    async fn ensure_window_creates_new() {
        let window = ensure_window(&MockNoExistingWindow, "sess", "my-win")
            .await
            .unwrap();
        assert_eq!(window.name, "my-win");
        assert_eq!(window.index, 1);
    }

    /// Mock where window already exists, so we return it without creating.
    struct MockExistingWindow;

    impl TmuxExecutor for MockExistingWindow {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            match args.first() {
                Some(&"list-windows") => Ok(TmuxOutput {
                    stdout: "@0\t0\tbash\t1\t1\n@2\t2\tmy-win\t3\t0\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                Some(&"new-window") => panic!("should not create when window exists"),
                _ => panic!("unexpected command: {args:?}"),
            }
        }
    }

    #[tokio::test]
    async fn ensure_window_returns_existing() {
        let window = ensure_window(&MockExistingWindow, "sess", "my-win")
            .await
            .unwrap();
        assert_eq!(window.name, "my-win");
        assert_eq!(window.index, 2);
        assert_eq!(window.panes, 3);
    }

    #[tokio::test]
    async fn ensure_window_validates_session() {
        let result = ensure_window(&MockNoExistingWindow, "", "win").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn ensure_window_validates_name() {
        let result = ensure_window(&MockNoExistingWindow, "sess", "").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }
}
