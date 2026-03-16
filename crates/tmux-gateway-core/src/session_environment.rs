use crate::executor::TmuxExecutor;
use crate::validation::{validate_env_var_name, validate_env_var_value, validate_session_target};

use super::TmuxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxEnvVar {
    pub name: String,
    pub value: Option<String>, // None means inherited/unset marker
}

impl std::fmt::Display for TmuxEnvVar {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Some(v) => write!(f, "{}={}", self.name, v),
            None => write!(f, "-{}", self.name),
        }
    }
}

/// Parse a single line of `show-environment -t` output into a `TmuxEnvVar`.
/// Lines are in the format `NAME=VALUE` or `-NAME` (unset marker).
pub(crate) fn parse_session_env_line(line: &str) -> Option<TmuxEnvVar> {
    if line.is_empty() {
        return None;
    }
    if let Some(name) = line.strip_prefix('-') {
        if name.is_empty() {
            return None;
        }
        return Some(TmuxEnvVar {
            name: name.to_string(),
            value: None,
        });
    }
    let (name, value) = line.split_once('=')?;
    Some(TmuxEnvVar {
        name: name.to_string(),
        value: Some(value.to_string()),
    })
}

/// List all environment variables for a session.
#[tracing::instrument(skip(executor))]
pub async fn show_environment(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
) -> Result<Vec<TmuxEnvVar>, TmuxError> {
    validate_session_target(session)?;
    let output = executor
        .execute(&["show-environment", "-t", session])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "show-environment",
            &output.stderr,
            session,
        ));
    }

    let vars = output
        .stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(parse_session_env_line)
        .collect();

    Ok(vars)
}

/// Set an environment variable for a session.
#[tracing::instrument(skip(executor))]
pub async fn set_environment(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
    name: &str,
    value: &str,
) -> Result<(), TmuxError> {
    validate_session_target(session)?;
    validate_env_var_name(name)?;
    validate_env_var_value(value)?;
    let output = executor
        .execute(&["set-environment", "-t", session, name, value])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "set-environment",
            &output.stderr,
            session,
        ));
    }
    Ok(())
}

/// Remove an environment variable from a session.
#[tracing::instrument(skip(executor))]
pub async fn unset_environment(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
    name: &str,
) -> Result<(), TmuxError> {
    validate_session_target(session)?;
    validate_env_var_name(name)?;
    let output = executor
        .execute(&["set-environment", "-t", session, "-u", name])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "set-environment",
            &output.stderr,
            session,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    struct MockExecutor {
        result: Result<TmuxOutput, TmuxError>,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            match &self.result {
                Ok(output) => Ok(output.clone()),
                Err(e) => Err(TmuxError::CommandFailed {
                    command: "mock".to_string(),
                    stderr: e.to_string(),
                }),
            }
        }
    }

    // ── parse_session_env_line ──

    #[test]
    fn parse_env_line_valid() {
        let env = parse_session_env_line("PATH=/usr/bin:/usr/local/bin").unwrap();
        assert_eq!(env.name, "PATH");
        assert_eq!(env.value, Some("/usr/bin:/usr/local/bin".to_string()));
    }

    #[test]
    fn parse_env_line_empty_value() {
        let env = parse_session_env_line("MY_VAR=").unwrap();
        assert_eq!(env.name, "MY_VAR");
        assert_eq!(env.value, Some(String::new()));
    }

    #[test]
    fn parse_env_line_value_with_equals() {
        let env = parse_session_env_line("FOO=bar=baz").unwrap();
        assert_eq!(env.name, "FOO");
        assert_eq!(env.value, Some("bar=baz".to_string()));
    }

    #[test]
    fn parse_env_line_unset_marker() {
        let env = parse_session_env_line("-REMOVED_VAR").unwrap();
        assert_eq!(env.name, "REMOVED_VAR");
        assert_eq!(env.value, None);
    }

    #[test]
    fn parse_env_line_no_equals() {
        assert!(parse_session_env_line("NOEQUALS").is_none());
    }

    #[test]
    fn parse_env_line_empty() {
        assert!(parse_session_env_line("").is_none());
    }

    #[test]
    fn parse_env_line_dash_only() {
        assert!(parse_session_env_line("-").is_none());
    }

    // ── TmuxEnvVar Display ──

    #[test]
    fn env_var_display_with_value() {
        let env = TmuxEnvVar {
            name: "EDITOR".to_string(),
            value: Some("vim".to_string()),
        };
        assert_eq!(env.to_string(), "EDITOR=vim");
    }

    #[test]
    fn env_var_display_unset() {
        let env = TmuxEnvVar {
            name: "REMOVED".to_string(),
            value: None,
        };
        assert_eq!(env.to_string(), "-REMOVED");
    }

    // ── show_environment ──

    #[tokio::test]
    async fn show_environment_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "PATH=/usr/bin\nEDITOR=vim\n-REMOVED\nTERM=screen\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let vars = show_environment(&executor, "my-session").await.unwrap();
        assert_eq!(vars.len(), 4);
        assert_eq!(vars[0].name, "PATH");
        assert_eq!(vars[0].value, Some("/usr/bin".to_string()));
        assert_eq!(vars[1].name, "EDITOR");
        assert_eq!(vars[1].value, Some("vim".to_string()));
        assert_eq!(vars[2].name, "REMOVED");
        assert_eq!(vars[2].value, None);
        assert_eq!(vars[3].name, "TERM");
        assert_eq!(vars[3].value, Some("screen".to_string()));
    }

    #[tokio::test]
    async fn show_environment_empty() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let vars = show_environment(&executor, "my-session").await.unwrap();
        assert!(vars.is_empty());
    }

    #[tokio::test]
    async fn show_environment_session_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: nonexistent".to_string(),
                success: false,
            }),
        };
        let result = show_environment(&executor, "nonexistent").await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn show_environment_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = show_environment(&executor, "my-session").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    #[tokio::test]
    async fn show_environment_invalid_session() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = show_environment(&executor, "").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    // ── set_environment ──

    #[tokio::test]
    async fn set_environment_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_environment(&executor, "my-session", "EDITOR", "vim").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn set_environment_invalid_session() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_environment(&executor, "", "EDITOR", "vim").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_environment_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_environment(&executor, "my-session", "bad name", "value").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_environment_invalid_value() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_environment(&executor, "my-session", "MY_VAR", "val\0ue").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_environment_session_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: nonexistent".to_string(),
                success: false,
            }),
        };
        let result = set_environment(&executor, "nonexistent", "EDITOR", "vim").await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn set_environment_command_failure() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = set_environment(&executor, "my-session", "EDITOR", "vim").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    // ── unset_environment ──

    #[tokio::test]
    async fn unset_environment_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = unset_environment(&executor, "my-session", "EDITOR").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn unset_environment_invalid_session() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = unset_environment(&executor, "", "EDITOR").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn unset_environment_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = unset_environment(&executor, "my-session", "").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn unset_environment_session_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: nonexistent".to_string(),
                success: false,
            }),
        };
        let result = unset_environment(&executor, "nonexistent", "EDITOR").await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn unset_environment_command_failure() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = unset_environment(&executor, "my-session", "EDITOR").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
