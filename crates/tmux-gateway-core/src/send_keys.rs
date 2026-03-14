use tmux_interface::{SendKeys, Tmux};

use super::TmuxError;
use super::validation::{ValidationError, validate_pane_target};

pub async fn send_keys(target: &str, keys: &[String]) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    if keys.is_empty() {
        return Err(ValidationError::EmptyInput { field: "keys" }.into());
    }
    let target = target.to_string();
    let keys = keys.to_vec();
    tokio::task::spawn_blocking(move || {
        let mut cmd = SendKeys::new().target_pane(target.as_str());
        for k in &keys {
            cmd = cmd.key(k.as_str());
        }

        let output = Tmux::with_command(cmd)
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "send-keys".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("send-keys", &stderr, &target));
        }

        Ok(())
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "send-keys".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
