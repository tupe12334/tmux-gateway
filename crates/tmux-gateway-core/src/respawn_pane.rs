use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::{validate_command, validate_pane_target};

/// Respawn a dead pane, optionally with a new command.
///
/// If `kill_existing` is `true`, the `-k` flag is passed to kill an active
/// pane before respawning it. Without `-k`, tmux will error when the pane
/// is still running.
#[tracing::instrument(skip(executor))]
pub async fn respawn_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    command: Option<&str>,
    kill_existing: bool,
) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }

    let mut args: Vec<&str> = vec!["respawn-pane"];
    if kill_existing {
        args.push("-k");
    }
    args.extend_from_slice(&["-t", target]);
    if let Some(cmd) = command {
        args.push(cmd);
    }

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "respawn-pane",
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
    async fn respawn_pane_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = respawn_pane(&executor, "sess:0.0", None, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_pane_with_command() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = respawn_pane(&executor, "sess:0.0", Some("bash"), false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_pane_kill_existing() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = respawn_pane(&executor, "sess:0.0", Some("bash"), true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_pane_invalid_target() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = respawn_pane(&executor, "", None, false).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn respawn_pane_invalid_command() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = respawn_pane(&executor, "sess:0.0", Some("rm; evil"), false).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn respawn_pane_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "pane not found: sess:0.99".to_string(),
                success: false,
            }),
        };
        let result = respawn_pane(&executor, "sess:0.99", None, false).await;
        assert!(matches!(result, Err(TmuxError::PaneNotFound(_))));
    }

    #[tokio::test]
    async fn respawn_pane_server_not_running() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            }),
        };
        let result = respawn_pane(&executor, "sess:0.0", None, false).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
