use crate::executor::{exec_tmux, run_tmux};
use crate::validation::validate_pane_target;
use tmux_interface::KillPane as TmuxKillPane;

use super::TmuxError;

pub async fn kill_pane(target: &str) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let target = target.to_string();
    run_tmux("kill-pane", move || {
        exec_tmux(
            "kill-pane",
            &target,
            TmuxKillPane::new().target_pane(target.as_str()),
        )?;
        Ok(())
    })
    .await
}
