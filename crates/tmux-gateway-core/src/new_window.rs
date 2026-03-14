use tmux_interface::{NewWindow, Tmux};

use super::TmuxError;
use super::validation::{validate_session_target, validate_window_name};
use crate::TmuxWindow;

pub async fn new_window(session: &str, name: &str) -> Result<TmuxWindow, TmuxError> {
    validate_session_target(session)?;
    validate_window_name(name)?;
    let session = session.to_string();
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            NewWindow::new()
                .detached()
                .target_window(session.as_str())
                .window_name(name.as_str())
                .print()
                .format("#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}"),
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() < 4 {
            return Err(TmuxError::ParseError {
                command: "new-window".to_string(),
                details: format!("expected 4 tab-separated fields, got: {line}"),
            });
        }
        let index = parts[0].parse::<u32>().map_err(|e| TmuxError::ParseError {
            command: "new-window".to_string(),
            details: format!("invalid window index '{}': {e}", parts[0]),
        })?;
        let panes = parts[2].parse::<u32>().map_err(|e| TmuxError::ParseError {
            command: "new-window".to_string(),
            details: format!("invalid pane count '{}': {e}", parts[2]),
        })?;
        Ok(TmuxWindow {
            index,
            name: parts[1].to_string(),
            panes,
            active: parts[3] == "1",
        })
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "new-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
