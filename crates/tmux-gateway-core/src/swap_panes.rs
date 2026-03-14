use tmux_interface::SwapPane;

use super::TmuxError;
use crate::executor::{exec_tmux, run_tmux};
use crate::validation::validate_pane_target;

/// Swap two panes by their targets (format: `session:window.pane`).
pub async fn swap_panes(src: &str, dst: &str) -> Result<(), TmuxError> {
    validate_pane_target(src)?;
    validate_pane_target(dst)?;
    let src = src.to_string();
    let dst = dst.to_string();
    run_tmux("swap-pane", move || {
        exec_tmux(
            "swap-pane",
            &src,
            SwapPane::new()
                .src_pane(src.as_str())
                .dst_pane(dst.as_str()),
        )?;
        Ok(())
    })
    .await
}
