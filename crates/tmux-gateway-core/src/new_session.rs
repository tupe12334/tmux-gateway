use crate::TmuxSession;
use crate::validation::validate_session_name;
use tmux_interface::{NewSession, Tmux};

use super::TmuxError;

pub async fn new_session(name: &str) -> Result<TmuxSession, TmuxError> {
    validate_session_name(name)?;
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            NewSession::new()
                .detached()
                .session_name(name.as_str())
                .print()
                .format(
                    "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
                ),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "new-session".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("new-session", &stderr, &name));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() < 4 {
            return Err(TmuxError::ParseError {
                command: "new-session".to_string(),
                details: format!("expected 4 tab-separated fields, got: {line}"),
            });
        }
        let windows = parts[1].parse::<u32>().map_err(|e| TmuxError::ParseError {
            command: "new-session".to_string(),
            details: format!("invalid window count '{}': {e}", parts[1]),
        })?;
        let created = parts[2].parse::<i64>().map_err(|e| TmuxError::ParseError {
            command: "new-session".to_string(),
            details: format!("invalid session_created timestamp '{}': {e}", parts[2]),
        })?;
        Ok(TmuxSession {
            name: parts[0].to_string(),
            windows,
            created,
            attached: parts[3] == "1",
        })
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "new-session".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
