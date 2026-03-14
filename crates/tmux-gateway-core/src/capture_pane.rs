use crate::executor::{exec_tmux, run_tmux};
use tmux_interface::CapturePane;

use super::TmuxError;

pub async fn capture_pane(target: &str) -> Result<String, TmuxError> {
    let target = target.to_string();
    run_tmux("capture-pane", move || {
        let output = exec_tmux(
            "capture-pane",
            &target,
            CapturePane::new().stdout().target_pane(target.as_str()),
        )?;
        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(content)
    })
    .await
}
