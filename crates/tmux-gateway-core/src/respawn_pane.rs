use super::TmuxError;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::{PaneTarget, validate_command};

/// Pure: build the tmux command specification for respawning a pane.
pub fn build_respawn_pane_command(
    target: &PaneTarget,
    command: Option<&str>,
    kill_existing: bool,
) -> TmuxCommandSpec {
    let mut args: Vec<String> = vec!["respawn-pane".into()];
    if kill_existing {
        args.push("-k".into());
    }
    args.extend_from_slice(&["-t".into(), target.as_str().into()]);
    if let Some(cmd) = command {
        args.push(cmd.into());
    }
    TmuxCommandSpec::new(args)
}

/// Respawn a dead pane, optionally with a new command.
///
/// If `kill_existing` is `true`, the `-k` flag is passed to kill an active
/// pane before respawning it. Without `-k`, tmux will error when the pane
/// is still running.
///
/// [tmux docs](https://man.openbsd.org/tmux#respawn-pane)
#[tracing::instrument(skip(executor))]
pub async fn respawn_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    command: Option<&str>,
    kill_existing: bool,
) -> Result<(), TmuxError> {
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }
    let spec = build_respawn_pane_command(target, command, kill_existing);
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
    async fn respawn_pane_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = respawn_pane(&executor, &target, None, false).await;
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
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = respawn_pane(&executor, &target, Some("bash"), false).await;
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
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = respawn_pane(&executor, &target, Some("bash"), true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn respawn_pane_invalid_target() {
        // With newtypes, invalid targets are caught at construction time
        assert!(PaneTarget::try_from("").is_err());
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
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = respawn_pane(&executor, &target, Some("rm; evil"), false).await;
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
        let target = PaneTarget::try_from("sess:0.99").unwrap();
        let result = respawn_pane(&executor, &target, None, false).await;
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
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = respawn_pane(&executor, &target, None, false).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    #[test]
    fn build_respawn_pane_command_basic() {
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let spec = build_respawn_pane_command(&target, None, false);
        assert_eq!(spec.command_name(), "respawn-pane");
        assert_eq!(spec.args(), vec!["respawn-pane", "-t", "sess:0.0"]);
    }

    #[test]
    fn build_respawn_pane_command_with_kill_and_command() {
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let spec = build_respawn_pane_command(&target, Some("bash"), true);
        assert_eq!(
            spec.args(),
            vec!["respawn-pane", "-k", "-t", "sess:0.0", "bash"]
        );
    }
}
