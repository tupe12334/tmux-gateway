use tmux_interface::{SwapPane, Tmux};

use super::TmuxError;
use crate::validation::validate_pane_target;

/// Swap two panes by their targets (format: `session:window.pane`).
pub async fn swap_panes(src: &str, dst: &str) -> Result<(), TmuxError> {
    validate_pane_target(src)?;
    validate_pane_target(dst)?;
    let src = src.to_string();
    let dst = dst.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            SwapPane::new()
                .src_pane(src.as_str())
                .dst_pane(dst.as_str()),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "swap-pane".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("swap-pane", &stderr, &src));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "swap-pane".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
