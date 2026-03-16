use super::TmuxError;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::WindowTarget;

/// Pure: build the tmux command specification for swapping windows.
pub fn build_swap_window_command(src: &WindowTarget, dst: &WindowTarget) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "swap-window".into(),
        "-s".into(),
        src.as_str().into(),
        "-t".into(),
        dst.as_str().into(),
    ])
}

/// Swap two windows by their targets (format: `session:window`).
#[tracing::instrument(skip(executor))]
pub async fn swap_window(
    executor: &(impl TmuxExecutor + ?Sized),
    src: &WindowTarget,
    dst: &WindowTarget,
) -> Result<(), TmuxError> {
    let spec = build_swap_window_command(src, dst);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            src.as_str(),
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
    async fn swap_window_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let src = WindowTarget::try_from("sess:0").unwrap();
        let dst = WindowTarget::try_from("sess:1").unwrap();
        let result = swap_window(&executor, &src, &dst).await;
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
        let src = WindowTarget::try_from("sess1:0").unwrap();
        let dst = WindowTarget::try_from("sess2:1").unwrap();
        let result = swap_window(&executor, &src, &dst).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn swap_window_invalid_source() {
        // With newtypes, invalid targets are caught at construction time
        assert!(WindowTarget::try_from("").is_err());
    }

    #[tokio::test]
    async fn swap_window_invalid_destination() {
        assert!(WindowTarget::try_from("invalid").is_err());
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
        let src = WindowTarget::try_from("sess:0").unwrap();
        let dst = WindowTarget::try_from("sess:99").unwrap();
        let result = swap_window(&executor, &src, &dst).await;
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
        let src = WindowTarget::try_from("sess:0").unwrap();
        let dst = WindowTarget::try_from("sess:1").unwrap();
        let result = swap_window(&executor, &src, &dst).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    #[test]
    fn build_swap_window_command_produces_correct_args() {
        let src = WindowTarget::try_from("sess:0").unwrap();
        let dst = WindowTarget::try_from("sess:1").unwrap();
        let spec = build_swap_window_command(&src, &dst);
        assert_eq!(spec.command_name(), "swap-window");
        assert_eq!(
            spec.args(),
            vec!["swap-window", "-s", "sess:0", "-t", "sess:1"]
        );
    }
}
