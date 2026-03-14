use tmux_interface::Tmux;

use crate::error::TmuxError;

/// Run a blocking tmux operation on a dedicated thread, mapping join errors.
///
/// This eliminates the `spawn_blocking` + join-error boilerplate repeated
/// across every operation module.
pub(crate) async fn run_tmux<T: Send + 'static>(
    command_name: &'static str,
    f: impl FnOnce() -> Result<T, TmuxError> + Send + 'static,
) -> Result<T, TmuxError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| TmuxError::CommandFailed {
            command: command_name.to_string(),
            stderr: format!("task join error: {e}"),
        })?
}

/// Execute a tmux command and check its exit status.
///
/// On failure, classifies the stderr into the appropriate `TmuxError` variant
/// using `TmuxError::from_stderr`. Returns the raw `Output` on success so
/// callers can read stdout when needed.
pub(crate) fn exec_tmux<'a>(
    command_name: &str,
    target: &str,
    cmd: impl Into<tmux_interface::TmuxCommand<'a>>,
) -> Result<std::process::Output, TmuxError> {
    let output = Tmux::with_command(cmd)
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: command_name.to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TmuxError::from_stderr(command_name, &stderr, target));
    }

    Ok(output)
}
