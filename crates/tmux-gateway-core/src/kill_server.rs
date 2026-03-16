use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};

use super::TmuxError;

/// Shut down the tmux server, destroying all sessions.
/// Returns Ok(()) if the server was killed or was already not running.
#[tracing::instrument(skip(executor))]
pub async fn kill_server(executor: &(impl TmuxExecutor + ?Sized)) -> Result<(), TmuxError> {
    kill_server_with_log(executor, &NoopLog).await
}

/// Shut down the tmux server with domain-level logging.
/// Returns Ok(()) if the server was killed or was already not running.
#[tracing::instrument(skip(executor, log))]
pub async fn kill_server_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    log: &dyn LogPort,
) -> Result<(), TmuxError> {
    log.log(LogLevel::Info, "kill-server", "shutting down tmux server");
    let output = executor.execute(&["kill-server"]).await?;
    if !output.success {
        // Idempotent: if the server is already not running, treat as success
        let stderr = output.stderr.to_lowercase();
        if stderr.contains("no server running") || stderr.contains("no current client") {
            log.log(
                LogLevel::Info,
                "kill-server",
                "tmux server was already not running",
            );
            return Ok(());
        }
        let err = TmuxError::CommandFailed {
            command: "kill-server".to_string(),
            stderr: output.stderr,
        };
        log.log(
            LogLevel::Error,
            "kill-server",
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    log.log(
        LogLevel::Info,
        "kill-server",
        "tmux server shut down successfully",
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
    async fn kill_server_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = kill_server(&executor).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn kill_server_already_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = kill_server(&executor).await;
        assert!(
            result.is_ok(),
            "should be idempotent when server is already down"
        );
    }

    #[tokio::test]
    async fn kill_server_unknown_error() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "some unexpected error".to_string(),
                success: false,
            }),
        };
        let result = kill_server(&executor).await;
        assert!(matches!(result, Err(TmuxError::CommandFailed { .. })));
    }

    #[tokio::test]
    async fn kill_server_executor_error() {
        let executor = MockExecutor {
            result: Err(TmuxError::Timeout {
                command: "kill-server".to_string(),
                timeout: std::time::Duration::from_secs(5),
            }),
        };
        let result = kill_server(&executor).await;
        assert!(matches!(result, Err(TmuxError::CommandFailed { .. })));
    }
}
