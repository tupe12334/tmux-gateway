use crate::executor::{exec_tmux, run_tmux};
use crate::validation::validate_window_target;
use tmux_interface::KillWindow as TmuxKillWindow;

use super::TmuxError;

pub async fn kill_window(target: &str) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    let target = target.to_string();
    run_tmux("kill-window", move || {
        exec_tmux(
            "kill-window",
            &target,
            TmuxKillWindow::new().target_window(target.as_str()),
        )?;
        Ok(())
    })
    .await
}
