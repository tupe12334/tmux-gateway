use crate::executor::{exec_tmux, run_tmux};
use crate::validation::validate_pane_target;
use tmux_interface::SelectPane as TmuxSelectPane;

use super::TmuxError;

pub async fn select_pane(target: &str) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let target = target.to_string();
    run_tmux("select-pane", move || {
        exec_tmux(
            "select-pane",
            &target,
            TmuxSelectPane::new().target_pane(target.as_str()),
        )?;
        Ok(())
    })
    .await
}
