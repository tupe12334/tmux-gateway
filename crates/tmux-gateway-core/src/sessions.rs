use serde::Serialize;
use tmux_interface::{ListSessions, Tmux};

use super::TmuxError;

#[derive(Debug, Clone, Serialize)]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub created: i64,
    pub attached: bool,
}

pub async fn session_exists(name: &str) -> Result<bool, TmuxError> {
    let sessions = list_sessions().await?;
    Ok(sessions.iter().any(|s| s.name == name))
}

pub async fn get_session(name: &str) -> Result<Option<TmuxSession>, TmuxError> {
    let sessions = list_sessions().await?;
    Ok(sessions.into_iter().find(|s| s.name == name))
}

pub async fn list_sessions() -> Result<Vec<TmuxSession>, TmuxError> {
    tokio::task::spawn_blocking(|| {
        let output = Tmux::with_command(ListSessions::new().format(
            "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
        ))
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "list-sessions".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no server running") || stderr.contains("no sessions") {
                return Ok(vec![]);
            }
            return Err(TmuxError::from_stderr("list-sessions", &stderr, ""));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let sessions = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 4 {
                    return Err(TmuxError::ParseError {
                        command: "list-sessions".to_string(),
                        details: format!("expected 4 tab-separated fields, got: {line}"),
                    });
                }
                let windows = parts[1].parse::<u32>().map_err(|e| TmuxError::ParseError {
                    command: "list-sessions".to_string(),
                    details: format!("invalid window count '{}': {e}", parts[1]),
                })?;
                let created = parts[2].parse::<i64>().map_err(|e| TmuxError::ParseError {
                    command: "list-sessions".to_string(),
                    details: format!("invalid session_created timestamp '{}': {e}", parts[2]),
                })?;
                Ok(TmuxSession {
                    name: parts[0].to_string(),
                    windows,
                    created,
                    attached: parts[3] == "1",
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "list-sessions".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn session_exists_returns_false_for_nonexistent() {
        let result = session_exists("__tmux_gw_test_nonexistent_session__").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn get_session_returns_none_for_nonexistent() {
        let result = get_session("__tmux_gw_test_nonexistent_session__").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn session_exists_finds_created_session() {
        let name = "__tmux_gw_test_exists__";
        // Create a detached session for testing
        let _ = Tmux::with_command(
            tmux_interface::NewSession::new()
                .detached()
                .session_name(name),
        )
        .output();

        let exists = session_exists(name).await.unwrap();
        assert!(exists);

        let session = get_session(name).await.unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().name, name);

        // Cleanup
        let _ =
            Tmux::with_command(tmux_interface::KillSession::new().target_session(name)).output();
    }
}
