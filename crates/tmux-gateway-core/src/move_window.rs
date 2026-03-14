use tmux_interface::{MoveWindow, Tmux};

use super::TmuxError;
use crate::validation::{validate_session_target, validate_window_target};

/// Move a window from one session to another.
///
/// `source` is in window target format (`session:window`).
/// `destination_session` is a session name.
pub async fn move_window(source: &str, destination_session: &str) -> Result<(), TmuxError> {
    validate_window_target(source)?;
    validate_session_target(destination_session)?;
    let source = source.to_string();
    let destination_session = destination_session.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            MoveWindow::new()
                .src_window(source.as_str())
                .dst_window(destination_session.as_str()),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "move-window".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("move-window", &stderr, &source));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "move-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
