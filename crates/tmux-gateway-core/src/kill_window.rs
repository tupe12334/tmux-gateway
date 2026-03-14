use crate::validation::validate_window_target;
use tmux_interface::{KillWindow as TmuxKillWindow, Tmux};

use super::TmuxError;

pub async fn kill_window(target: &str) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(TmuxKillWindow::new().target_window(target.as_str()))
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "kill-window".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("kill-window", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "kill-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
