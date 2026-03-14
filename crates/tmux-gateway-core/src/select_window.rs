use crate::executor::{exec_tmux, run_tmux};
use crate::validation::validate_window_target;
use tmux_interface::SelectWindow as TmuxSelectWindow;

use super::TmuxError;

pub async fn select_window(target: &str) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    let target = target.to_string();
    run_tmux("select-window", move || {
        exec_tmux(
            "select-window",
            &target,
            TmuxSelectWindow::new().target_window(target.as_str()),
        )?;
        Ok(())
    })
    .await
}
