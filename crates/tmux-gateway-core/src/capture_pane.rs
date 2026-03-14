use tmux_interface::{CapturePane, Tmux};

use super::TmuxError;

pub async fn capture_pane(target: &str) -> Result<String, TmuxError> {
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            CapturePane::new()
                .stdout()
                .target_pane(target.as_str()),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "capture-pane".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("capture-pane", &stderr, &target));
        }

        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(content)
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "capture-pane".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
