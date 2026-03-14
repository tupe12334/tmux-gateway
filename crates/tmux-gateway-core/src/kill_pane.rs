use crate::validation::validate_pane_target;
use tmux_interface::{KillPane as TmuxKillPane, Tmux};

use super::TmuxError;

pub async fn kill_pane(target: &str) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(TmuxKillPane::new().target_pane(target.as_str()))
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "kill-pane".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("kill-pane", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "kill-pane".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
