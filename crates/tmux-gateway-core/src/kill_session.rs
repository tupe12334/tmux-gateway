use crate::executor::{exec_tmux, run_tmux};
use crate::validation::validate_session_target;
use tmux_interface::KillSession as TmuxKillSession;

use super::TmuxError;

pub async fn kill_session(target: &str) -> Result<(), TmuxError> {
    validate_session_target(target)?;
    let target = target.to_string();
    run_tmux("kill-session", move || {
        exec_tmux(
            "kill-session",
            &target,
            TmuxKillSession::new().target_session(target.as_str()),
        )?;
        Ok(())
    })
    .await
}
