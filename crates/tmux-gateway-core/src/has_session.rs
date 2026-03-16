use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::validation::SessionName;

use super::TmuxError;

/// Check if a session exists using tmux `has-session` (O(1) exit-code check).
#[tracing::instrument(skip(executor))]
pub async fn has_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
) -> Result<bool, TmuxError> {
    has_session_with_log(executor, target, &NoopLog).await
}

/// Check if a session exists with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn has_session_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
    log: &dyn LogPort,
) -> Result<bool, TmuxError> {
    let target_str = target.as_str();
    log.log_with_target(
        LogLevel::Debug,
        "has-session",
        target_str,
        &format!("checking if session '{target_str}' exists"),
    );
    let output = executor
        .execute(&["has-session", "-t", target_str])
        .await?;
    if output.success {
        log.log_with_target(
            LogLevel::Debug,
            "has-session",
            target_str,
            &format!("session '{target_str}' exists"),
        );
        return Ok(true);
    }
    // "no server running" means tmux isn't up — propagate as error
    if output.stderr.contains("no server running") {
        let err = TmuxError::TmuxNotRunning;
        log.log_with_target(
            LogLevel::Error,
            "has-session",
            target_str,
            &format!("tmux server not running: {err}"),
        );
        return Err(err);
    }
    // Any other failure (e.g. "session not found") means the session doesn't exist
    log.log_with_target(
        LogLevel::Debug,
        "has-session",
        target_str,
        &format!("session '{target_str}' does not exist"),
    );
    Ok(false)
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

    #[tokio::test]
    async fn has_session_returns_true_when_exists() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let name = SessionName::try_from("my-session").unwrap();
        assert!(has_session(&executor, &name).await.unwrap());
    }

    #[tokio::test]
    async fn has_session_returns_false_when_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "can't find session: nosession".to_string(),
                success: false,
            }),
        };
        let name = SessionName::try_from("nosession").unwrap();
        assert!(!has_session(&executor, &name).await.unwrap());
    }

    #[tokio::test]
    async fn has_session_returns_error_when_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let name = SessionName::try_from("test-session").unwrap();
        let result = has_session(&executor, &name).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    #[tokio::test]
    async fn has_session_rejects_empty_target() {
        // With newtypes, validation happens at construction time
        assert!(SessionName::try_from("").is_err());
    }

    #[tokio::test]
    async fn has_session_returns_false_on_generic_failure() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: gone".to_string(),
                success: false,
            }),
        };
        let name = SessionName::try_from("gone").unwrap();
        assert!(!has_session(&executor, &name).await.unwrap());
    }

    #[tokio::test]
    async fn has_session_with_valid_hyphenated_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let name = SessionName::try_from("my-test-session").unwrap();
        assert!(has_session(&executor, &name).await.unwrap());
    }
}
