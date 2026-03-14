use crate::validation::validate_pane_target;
use tmux_interface::{KillPane as TmuxKillPane, Tmux};

use super::TmuxError;
use crate::timeout::spawn_blocking_with_timeout;

pub async fn kill_pane(target: &str) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let target = target.to_string();
    spawn_blocking_with_timeout("kill-pane", move || {
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
}
