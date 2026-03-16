use super::TmuxError;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::{WindowTarget, validate_command};

/// Pure: build the tmux command specification for respawning a window.
pub fn build_respawn_window_command(
    target: &WindowTarget,
    command: Option<&str>,
    kill_existing: bool,
) -> TmuxCommandSpec {
    let mut args: Vec<String> = vec!["respawn-window".into()];
    if kill_existing {
        args.push("-k".into());
    }
    args.extend_from_slice(&["-t".into(), target.as_str().into()]);
    if let Some(cmd) = command {
        args.push(cmd.into());
    }
    TmuxCommandSpec::new(args)
}

/// Respawn a dead window, optionally with a new command.
///
/// If `kill_existing` is `true`, the `-k` flag is passed to kill an active
/// window before respawning it. Without `-k`, tmux will error when the
/// window is still running.
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
    let spec = build_respawn_window_command(target, command, kill_existing);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            target.as_str(),
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

    #[test]
    fn build_respawn_window_command_basic() {
        let target = WindowTarget::try_from("sess:0").unwrap();
        let spec = build_respawn_window_command(&target, None, false);
        assert_eq!(spec.command_name(), "respawn-window");
        assert_eq!(spec.args(), vec!["respawn-window", "-t", "sess:0"]);
    }

    #[test]
    fn build_respawn_window_command_with_kill_and_command() {
        let target = WindowTarget::try_from("sess:0").unwrap();
        let spec = build_respawn_window_command(&target, Some("bash"), true);
        assert_eq!(
            spec.args(),
            vec!["respawn-window", "-k", "-t", "sess:0", "bash"]
        );
    }
}
