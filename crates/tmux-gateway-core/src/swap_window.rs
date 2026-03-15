use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::validate_window_target;

/// Swap two windows by their targets (format: `session:window`).
#[tracing::instrument(skip(executor))]
pub async fn swap_window(
    executor: &(impl TmuxExecutor + ?Sized),
    src: &str,
    dst: &str,
) -> Result<(), TmuxError> {
    validate_window_target(src)?;
    validate_window_target(dst)?;
    let output = executor
        .execute(&["swap-window", "-s", src, "-t", dst])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr("swap-window", &output.stderr, src));
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
    async fn swap_window_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = swap_window(&executor, "sess:0", "sess:1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn swap_window_cross_session() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = swap_window(&executor, "sess1:0", "sess2:1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn swap_window_invalid_source() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = swap_window(&executor, "", "sess:1").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn swap_window_invalid_destination() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = swap_window(&executor, "sess:0", "invalid").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn swap_window_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "window not found: sess:99".to_string(),
                success: false,
            }),
        };
        let result = swap_window(&executor, "sess:0", "sess:99").await;
        assert!(matches!(result, Err(TmuxError::WindowNotFound(_))));
    }

    #[tokio::test]
    async fn swap_window_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = swap_window(&executor, "sess:0", "sess:1").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
