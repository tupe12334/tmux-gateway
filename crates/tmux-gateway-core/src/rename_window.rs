use tmux_interface::{RenameWindow as TmuxRenameWindow, Tmux};

use super::TmuxError;

pub async fn rename_window(target: &str, new_name: &str) -> Result<(), TmuxError> {
    let target = target.to_string();
    let new_name = new_name.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            TmuxRenameWindow::new()
                .target_window(target.as_str())
                .new_name(new_name.as_str()),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "rename-window".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("rename-window", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "rename-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
