use std::path::PathBuf;
use std::time::Duration;

use crate::TmuxError;
use crate::executor::{TmuxExecutor, TmuxOutput};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Returns the configured tmux command timeout from `TMUX_COMMAND_TIMEOUT_SECS`,
/// defaulting to 30 seconds.
fn command_timeout() -> Duration {
    let secs = std::env::var("TMUX_COMMAND_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Executor that spawns real `tmux` processes.
///
/// By default connects to the system-default tmux server socket.
/// Use [`RealTmuxExecutor::with_socket`] to target a specific server instance.
#[derive(Debug, Clone, Default)]
pub struct RealTmuxExecutor {
    pub socket_path: Option<PathBuf>,
}

impl RealTmuxExecutor {
    /// Create an executor that uses the default tmux server socket.
    pub fn new() -> Self {
        Self { socket_path: None }
    }

    /// Create an executor that targets a specific tmux server socket.
    pub fn with_socket(path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: Some(path.into()),
        }
    }
}

impl TmuxExecutor for RealTmuxExecutor {
    async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
        let socket_path = self.socket_path.clone();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let cmd_name = args.first().cloned().unwrap_or_default();
        let timeout_dur = command_timeout();
        let cmd_for_timeout = cmd_name.clone();
        tokio::time::timeout(
            timeout_dur,
            tokio::task::spawn_blocking(move || {
                let mut cmd = std::process::Command::new("tmux");
                if let Some(ref socket) = socket_path {
                    cmd.arg("-S").arg(socket);
                }
                let output = cmd
                    .args(&args)
                    .output()
                    .map_err(|e| TmuxError::CommandFailed {
                        command: cmd_name.clone(),
                        stderr: e.to_string(),
                    })?;
                Ok(TmuxOutput {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    success: output.status.success(),
                })
            }),
        )
        .await
        .map_err(|_| TmuxError::Timeout {
            command: cmd_for_timeout,
            timeout: timeout_dur,
        })?
        .map_err(|e| TmuxError::CommandFailed {
            command: "spawn_blocking".to_string(),
            stderr: format!("task join error: {e}"),
        })?
    }
}
