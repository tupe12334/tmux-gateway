use tmux_interface::{RenameSession as TmuxRenameSession, Tmux};

use super::TmuxError;

pub async fn rename_session(target: &str, new_name: &str) -> Result<(), TmuxError> {
    let target = target.to_string();
    let new_name = new_name.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            TmuxRenameSession::new()
                .target_session(target.as_str())
                .new_name(new_name.as_str()),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "rename-session".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("rename-session", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "rename-session".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
