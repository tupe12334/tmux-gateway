use tmux_interface::{SplitWindow, Tmux};

use super::TmuxError;

pub async fn split_window(target: &str, horizontal: bool) -> Result<(), TmuxError> {
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let mut cmd = SplitWindow::new().detached().target_pane(target.as_str());
        if horizontal {
            cmd = cmd.horizontal();
        } else {
            cmd = cmd.vertical();
        }

        let output = Tmux::with_command(cmd)
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "split-window".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("split-window", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "split-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
