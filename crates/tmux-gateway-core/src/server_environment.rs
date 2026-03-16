use crate::executor::TmuxExecutor;
use crate::validation::{validate_env_var_name, validate_env_var_value};

use super::TmuxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

impl std::fmt::Display for EnvVar {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

/// Parse a single line of `show-environment -g` output into an `EnvVar`.
/// Lines are in the format `NAME=VALUE`. Lines starting with `-` (unset markers) are skipped.
pub(crate) fn parse_env_line(line: &str) -> Option<EnvVar> {
    if line.starts_with('-') {
        return None;
    }
    let (name, value) = line.split_once('=')?;
    Some(EnvVar {
        name: name.to_string(),
        value: value.to_string(),
    })
}

/// List all server-level (global) environment variables.
#[tracing::instrument(skip(executor))]
pub async fn list_server_environment(
    executor: &(impl TmuxExecutor + ?Sized),
) -> Result<Vec<EnvVar>, TmuxError> {
    let output = executor.execute(&["show-environment", "-g"]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "show-environment",
            &output.stderr,
            "",
        ));
    }

    let vars = output
        .stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(parse_env_line)
        .collect();

    Ok(vars)
}

/// Get a specific server-level environment variable by name.
#[tracing::instrument(skip(executor))]
pub async fn get_server_env(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<Option<String>, TmuxError> {
    validate_env_var_name(name)?;
    let output = executor.execute(&["show-environment", "-g", name]).await?;
    if !output.success {
        let stderr = output.stderr.to_lowercase();
        if stderr.contains("unknown variable") {
            return Ok(None);
        }
        return Err(TmuxError::from_stderr(
            "show-environment",
            &output.stderr,
            name,
        ));
    }

    let line = output.stdout.trim();
    if line.is_empty() {
        return Ok(None);
    }

    Ok(parse_env_line(line).map(|env| env.value))
}

/// Set a server-level environment variable.
#[tracing::instrument(skip(executor))]
pub async fn set_server_env(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
    value: &str,
) -> Result<(), TmuxError> {
    validate_env_var_name(name)?;
    validate_env_var_value(value)?;
    let output = executor
        .execute(&["set-environment", "-g", name, value])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "set-environment",
            &output.stderr,
            name,
        ));
    }
    Ok(())
}

/// Remove a server-level environment variable.
#[tracing::instrument(skip(executor))]
pub async fn unset_server_env(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<(), TmuxError> {
    validate_env_var_name(name)?;
    let output = executor
        .execute(&["set-environment", "-g", "-u", name])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "set-environment",
            &output.stderr,
            name,
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

    // ── parse_env_line ──

    #[test]
    fn parse_env_line_valid() {
        let env = parse_env_line("PATH=/usr/bin:/usr/local/bin").unwrap();
        assert_eq!(env.name, "PATH");
        assert_eq!(env.value, "/usr/bin:/usr/local/bin");
    }

    #[test]
    fn parse_env_line_empty_value() {
        let env = parse_env_line("MY_VAR=").unwrap();
        assert_eq!(env.name, "MY_VAR");
        assert_eq!(env.value, "");
    }

    #[test]
    fn parse_env_line_value_with_equals() {
        let env = parse_env_line("FOO=bar=baz").unwrap();
        assert_eq!(env.name, "FOO");
        assert_eq!(env.value, "bar=baz");
    }

    #[test]
    fn parse_env_line_unset_marker() {
        assert!(parse_env_line("-REMOVED_VAR").is_none());
    }

    #[test]
    fn parse_env_line_no_equals() {
        assert!(parse_env_line("NOEQUALS").is_none());
    }

    // ── EnvVar Display ──

    #[test]
    fn env_var_display() {
        let env = EnvVar {
            name: "EDITOR".to_string(),
            value: "vim".to_string(),
        };
        assert_eq!(env.to_string(), "EDITOR=vim");
    }

    // ── list_server_environment ──

    #[tokio::test]
    async fn list_server_environment_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "PATH=/usr/bin\nEDITOR=vim\nTERM=screen-256color\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let vars = list_server_environment(&executor).await.unwrap();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].name, "PATH");
        assert_eq!(vars[0].value, "/usr/bin");
        assert_eq!(vars[1].name, "EDITOR");
        assert_eq!(vars[1].value, "vim");
        assert_eq!(vars[2].name, "TERM");
        assert_eq!(vars[2].value, "screen-256color");
    }

    #[tokio::test]
    async fn list_server_environment_skips_unset_markers() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "PATH=/usr/bin\n-REMOVED_VAR\nEDITOR=vim\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let vars = list_server_environment(&executor).await.unwrap();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].name, "PATH");
        assert_eq!(vars[1].name, "EDITOR");
    }

    #[tokio::test]
    async fn list_server_environment_empty() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let vars = list_server_environment(&executor).await.unwrap();
        assert!(vars.is_empty());
    }

    #[tokio::test]
    async fn list_server_environment_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = list_server_environment(&executor).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    // ── get_server_env ──

    #[tokio::test]
    async fn get_server_env_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "EDITOR=vim\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let val = get_server_env(&executor, "EDITOR").await.unwrap();
        assert_eq!(val, Some("vim".to_string()));
    }

    #[tokio::test]
    async fn get_server_env_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "unknown variable: NONEXISTENT".to_string(),
                success: false,
            }),
        };
        let val = get_server_env(&executor, "NONEXISTENT").await.unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn get_server_env_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = get_server_env(&executor, "").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn get_server_env_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = get_server_env(&executor, "PATH").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    // ── set_server_env ──

    #[tokio::test]
    async fn set_server_env_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_server_env(&executor, "EDITOR", "vim").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn set_server_env_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_server_env(&executor, "bad name", "value").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_server_env_invalid_value() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_server_env(&executor, "MY_VAR", "val\0ue").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_server_env_command_failure() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = set_server_env(&executor, "EDITOR", "vim").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    // ── unset_server_env ──

    #[tokio::test]
    async fn unset_server_env_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = unset_server_env(&executor, "EDITOR").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn unset_server_env_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = unset_server_env(&executor, "").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn unset_server_env_command_failure() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = unset_server_env(&executor, "EDITOR").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
