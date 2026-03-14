use crate::validation::validate_session_target;
use tmux_interface::{KillSession as TmuxKillSession, Tmux};

use super::TmuxError;

pub async fn kill_session(target: &str) -> Result<(), TmuxError> {
    validate_session_target(target)?;
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(TmuxKillSession::new().target_session(target.as_str()))
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "kill-session".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("kill-session", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "kill-session".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
