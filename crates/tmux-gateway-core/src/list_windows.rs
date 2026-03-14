use serde::Serialize;
use tmux_interface::{ListWindows, Tmux};

use super::TmuxError;
use super::validation::validate_session_target;

#[derive(Debug, Clone, Serialize)]
pub struct TmuxWindow {
    pub index: u32,
    pub name: String,
    pub panes: u32,
    pub active: bool,
}

pub async fn list_windows(session: &str) -> Result<Vec<TmuxWindow>, TmuxError> {
    validate_session_target(session)?;
    let session = session.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            ListWindows::new()
                .target_session(session.as_str())
                .format("#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}"),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "list-windows".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("list-windows", &stderr, &session));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let windows = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 4 {
                    return Err(TmuxError::ParseError {
                        command: "list-windows".to_string(),
                        details: format!("expected 4 tab-separated fields, got: {line}"),
                    });
                }
                let index = parts[0].parse::<u32>().map_err(|e| TmuxError::ParseError {
                    command: "list-windows".to_string(),
                    details: format!("invalid window index '{}': {e}", parts[0]),
                })?;
                let panes = parts[2].parse::<u32>().map_err(|e| TmuxError::ParseError {
                    command: "list-windows".to_string(),
                    details: format!("invalid pane count '{}': {e}", parts[2]),
                })?;
                Ok(TmuxWindow {
                    index,
                    name: parts[1].to_string(),
                    panes,
                    active: parts[3] == "1",
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(windows)
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "list-windows".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
