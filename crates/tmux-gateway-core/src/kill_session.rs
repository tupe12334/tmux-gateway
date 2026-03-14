use crate::executor::TmuxExecutor;
use crate::validation::validate_session_target;

use super::TmuxError;

pub async fn kill_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<(), TmuxError> {
    validate_session_target(target)?;
    let output = executor.execute(&["kill-session", "-t", target]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "kill-session",
            &output.stderr,
            target,
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

    #[tokio::test]
    async fn kill_session_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = kill_session(&executor, "test-session").await;
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
        let result = kill_session(&executor, "nosession").await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn kill_session_invalid_target() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = kill_session(&executor, "").await;
        assert!(matches!(result, Err(TmuxError::InvalidTarget(_))));
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
        let result = kill_session(&executor, "test-session").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
