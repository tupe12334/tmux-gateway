use tmux_interface::{NewWindow, Tmux};

use super::TmuxError;
use super::validation::{validate_session_target, validate_window_name};

pub async fn new_window(session: &str, name: &str) -> Result<String, TmuxError> {
    validate_session_target(session)?;
    validate_window_name(name)?;
    let session = session.to_string();
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            NewWindow::new()
                .detached()
                .target_window(session.as_str())
                .window_name(name.as_str()),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "new-window".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("new-window", &stderr, &session));
        }

        Ok(name)
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "new-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
