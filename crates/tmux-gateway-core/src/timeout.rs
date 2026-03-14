use std::time::Duration;

use tokio::time;

use crate::TmuxError;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Returns the configured tmux command timeout from `TMUX_COMMAND_TIMEOUT_SECS`,
/// defaulting to 30 seconds.
pub fn command_timeout() -> Duration {
    let secs = std::env::var("TMUX_COMMAND_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Wraps a `spawn_blocking` call with the configured timeout.
/// Returns `TmuxError::Timeout` if the operation exceeds the deadline.
pub async fn spawn_blocking_with_timeout<F, T>(command: &str, f: F) -> Result<T, TmuxError>
where
    F: FnOnce() -> Result<T, TmuxError> + Send + 'static,
    T: Send + 'static,
{
    let timeout_dur = command_timeout();
    let cmd = command.to_string();

    time::timeout(timeout_dur, tokio::task::spawn_blocking(f))
        .await
        .map_err(|_| TmuxError::Timeout {
            command: cmd.clone(),
        })?
        .map_err(|e| TmuxError::CommandFailed {
            command: cmd,
            stderr: format!("task join error: {e}"),
        })?
}
