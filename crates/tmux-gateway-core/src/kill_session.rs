use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::preconditions::require_session_exists;
use crate::validation::SessionName;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn kill_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
) -> Result<(), TmuxError> {
    kill_session_with_log(executor, target, &NoopLog).await
}

/// Kill a session with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn kill_session_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
    log: &dyn LogPort,
) -> Result<(), TmuxError> {
    require_session_exists(executor, target).await?;
    let target_str = target.as_str();
    log.log_with_target(
        LogLevel::Info,
        "kill-session",
        target_str,
        &format!("killing session '{target_str}'"),
    );
    let output = executor
        .execute(&["kill-session", "-t", target_str])
        .await?;
    if !output.success {
        let err = TmuxError::from_stderr("kill-session", &output.stderr, target_str);
        log.log_with_target(
            LogLevel::Error,
            "kill-session",
            target_str,
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    log.log_with_target(
        LogLevel::Info,
        "kill-session",
        target_str,
        &format!("session '{target_str}' killed successfully"),
    );
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

    #[tokio::test]
    async fn kill_session_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let name = SessionName::try_from("test-session").unwrap();
        let result = kill_session(&executor, &name).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn kill_session_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: nosession".to_string(),
                success: false,
            }),
        };
        let name = SessionName::try_from("nosession").unwrap();
        let result = kill_session(&executor, &name).await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn kill_session_invalid_target() {
        // With newtypes, invalid names are caught at construction time
        let result = SessionName::try_from("");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn kill_session_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let name = SessionName::try_from("test-session").unwrap();
        let result = kill_session(&executor, &name).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
