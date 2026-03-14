use serde::Serialize;
use tmux_interface::{ListSessions, Tmux};

use super::TmuxError;
use crate::executor::run_tmux;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TmuxSession {
    pub id: String,
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

pub(crate) fn parse_session_line(line: &str) -> Result<TmuxSession, TmuxError> {
    let parts: Vec<&str> = line.splitn(5, '\t').collect();
    if parts.len() < 5 {
        return Err(TmuxError::ParseError {
            command: "list-sessions".to_string(),
            details: format!("expected 5 tab-separated fields, got: {line}"),
        });
    }
    let windows = parts[2].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-sessions".to_string(),
        details: format!("invalid window count '{}': {e}", parts[2]),
    })?;
    let created = parts[3].parse::<i64>().map_err(|e| TmuxError::ParseError {
        command: "list-sessions".to_string(),
        details: format!("invalid session_created timestamp '{}': {e}", parts[3]),
    })?;
    Ok(TmuxSession {
        id: parts[0].to_string(),
        name: parts[1].to_string(),
        windows,
        created,
        attached: parts[4] == "1",
    })
}

pub(crate) fn parse_sessions(stdout: &str) -> Result<Vec<TmuxSession>, TmuxError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_session_line)
        .collect()
}

pub async fn list_sessions() -> Result<Vec<TmuxSession>, TmuxError> {
    run_tmux("list-sessions", || {
        let output = Tmux::with_command(ListSessions::new().format(
            "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
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
        parse_sessions(&stdout)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_line_valid() {
        let session = parse_session_line("$0\tmysession\t3\t1700000000\t1").unwrap();
        assert_eq!(session.id, "$0");
        assert_eq!(session.name, "mysession");
        assert_eq!(session.windows, 3);
        assert_eq!(session.created, 1700000000);
        assert!(session.attached);
    }

    #[test]
    fn parse_session_line_not_attached() {
        let session = parse_session_line("$1\tdev\t1\t1700000000\t0").unwrap();
        assert_eq!(session.id, "$1");
        assert!(!session.attached);
    }

    #[test]
    fn parse_session_line_missing_fields() {
        let result = parse_session_line("only\ttwo");
        assert!(result.is_err());
    }

    #[test]
    fn parse_session_line_invalid_window_count() {
        let result = parse_session_line("$0\ts\tnotanum\t1700000000\t0");
        assert!(result.is_err());
    }

    #[test]
    fn parse_session_line_invalid_timestamp() {
        let result = parse_session_line("$0\ts\t1\tbadts\t0");
        assert!(result.is_err());
    }

    #[test]
    fn parse_sessions_multiple_lines() {
        let input = "$0\ta\t1\t100\t0\n$1\tb\t2\t200\t1\n";
        let sessions = parse_sessions(input).unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, "$0");
        assert_eq!(sessions[0].name, "a");
        assert_eq!(sessions[1].id, "$1");
        assert_eq!(sessions[1].name, "b");
    }

    #[test]
    fn parse_sessions_empty_input() {
        let sessions = parse_sessions("").unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn parse_sessions_skips_empty_lines() {
        let input = "\n$0\ta\t1\t100\t0\n\n";
        let sessions = parse_sessions(input).unwrap();
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn parse_sessions_propagates_error() {
        let input = "$0\tgood\t1\t100\t0\nbad line";
        let result = parse_sessions(input);
        assert!(result.is_err());
    }

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
