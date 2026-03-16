use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::{WindowTarget, validate_command};

/// Respawn a dead window, optionally with a new command.
///
/// If `kill_existing` is `true`, the `-k` flag is passed to kill an active
/// window before respawning it. Without `-k`, tmux will error when the
/// window is still running.
///
/// [tmux docs](https://man.openbsd.org/tmux#respawn-window)
#[tracing::instrument(skip(executor))]
pub async fn respawn_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
    command: Option<&str>,
    kill_existing: bool,
) -> Result<(), TmuxError> {
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }

    let target_str = target.as_str();
    let mut args: Vec<&str> = vec!["respawn-window"];
    if kill_existing {
        args.push("-k");
    }
    args.extend_from_slice(&["-t", target_str]);
    if let Some(cmd) = command {
        args.push(cmd);
    }

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "respawn-window",
            &output.stderr,
            target_str,
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
    async fn respawn_window_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let target = WindowTarget::try_from("sess:0").unwrap();
        let result = respawn_window(&executor, &target, None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_window_with_command() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let target = WindowTarget::try_from("sess:0").unwrap();
        let result = respawn_window(&executor, &target, Some("bash"), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_window_kill_existing() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let target = WindowTarget::try_from("sess:0").unwrap();
        let result = respawn_window(&executor, &target, Some("bash"), true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_window_invalid_target() {
        assert!(WindowTarget::try_from("").is_err());
    }

    #[tokio::test]
    async fn respawn_window_invalid_command() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let target = WindowTarget::try_from("sess:0").unwrap();
        let result = respawn_window(&executor, &target, Some("rm; evil"), false).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn respawn_window_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "window not found: sess:99".to_string(),
                success: false,
            }),
        };
        let target = WindowTarget::try_from("sess:99").unwrap();
        let result = respawn_window(&executor, &target, None, false).await;
        assert!(matches!(result, Err(TmuxError::WindowNotFound(_))));
    }

    #[tokio::test]
    async fn respawn_window_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let target = WindowTarget::try_from("sess:0").unwrap();
        let result = respawn_window(&executor, &target, None, false).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
