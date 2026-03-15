use std::fmt;

use super::TmuxError;
use crate::executor::TmuxExecutor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxSession {
    pub id: String,
    pub name: String,
    pub windows: u32,
    pub created: i64,
    pub attached: bool,
}

impl fmt::Display for TmuxSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({} windows, {})",
            self.name,
            self.windows,
            if self.attached {
                "attached"
            } else {
                "detached"
            }
        )
    }
}

#[tracing::instrument(skip(executor))]
pub async fn session_exists(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<bool, TmuxError> {
    let sessions = list_sessions(executor).await?;
    Ok(sessions.iter().any(|s| s.name == name))
}

#[tracing::instrument(skip(executor))]
pub async fn get_session(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<Option<TmuxSession>, TmuxError> {
    let sessions = list_sessions(executor).await?;
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

#[tracing::instrument(skip(executor))]
pub async fn list_sessions(
    executor: &(impl TmuxExecutor + ?Sized),
) -> Result<Vec<TmuxSession>, TmuxError> {
    let output = executor
        .execute(&[
            "list-sessions",
            "-F",
            "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
        ])
        .await?;

    if !output.success {
        let stderr = &output.stderr;
        if stderr.contains("no server running") || stderr.contains("no sessions") {
            return Ok(vec![]);
        }
        return Err(TmuxError::from_stderr("list-sessions", stderr, ""));
    }

    parse_sessions(&output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{RealTmuxExecutor, TmuxOutput};

    #[test]
    fn display_attached_session() {
        let session = TmuxSession {
            id: "$0".to_string(),
            name: "dev".to_string(),
            windows: 3,
            created: 1700000000,
            attached: true,
        };
        assert_eq!(session.to_string(), "dev (3 windows, attached)");
    }

    #[test]
    fn display_detached_session() {
        let session = TmuxSession {
            id: "$1".to_string(),
            name: "prod".to_string(),
            windows: 1,
            created: 1700000000,
            attached: false,
        };
        assert_eq!(session.to_string(), "prod (1 windows, detached)");
    }

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
        let result =
            session_exists(&RealTmuxExecutor, "__tmux_gw_test_nonexistent_session__").await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn get_session_returns_none_for_nonexistent() {
        let result = get_session(&RealTmuxExecutor, "__tmux_gw_test_nonexistent_session__").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn session_exists_finds_created_session() {
        let name = "__tmux_gw_test_exists__";
        // Create a detached session for testing
        let _ = std::process::Command::new("tmux")
            .args(["new-session", "-d", "-s", name])
            .output();

        let exists = session_exists(&RealTmuxExecutor, name).await.unwrap();
        assert!(exists);

        let session = get_session(&RealTmuxExecutor, name).await.unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().name, name);

        // Cleanup
        let _ = std::process::Command::new("tmux")
            .args(["kill-session", "-t", name])
            .output();
    }

    // ── Mock executor tests ──

    struct MockExecutor {
        output: TmuxOutput,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            Ok(self.output.clone())
        }
    }

    #[tokio::test]
    async fn list_sessions_parses_mock_output() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "$0\tdev\t2\t1700000000\t0\n$1\tprod\t5\t1700000100\t1\n".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let sessions = list_sessions(&executor).await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, "$0");
        assert_eq!(sessions[0].name, "dev");
        assert_eq!(sessions[0].windows, 2);
        assert_eq!(sessions[1].id, "$1");
        assert_eq!(sessions[1].name, "prod");
        assert!(sessions[1].attached);
    }

    #[tokio::test]
    async fn list_sessions_returns_empty_on_no_server() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            },
        };
        let sessions = list_sessions(&executor).await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn session_exists_with_mock() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "$0\tmysess\t1\t100\t0\n".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        assert!(session_exists(&executor, "mysess").await.unwrap());
        assert!(!session_exists(&executor, "other").await.unwrap());
    }
}
