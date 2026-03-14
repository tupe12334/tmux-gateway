use crate::executor::{exec_tmux, run_tmux};
use crate::validation::{ValidationError, validate_pane_target};
use tmux_interface::SendKeys;

use super::TmuxError;

pub async fn send_keys(target: &str, keys: &[String]) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    if keys.is_empty() {
        return Err(ValidationError::EmptyInput { field: "keys" }.into());
    }
    let target = target.to_string();
    let keys = keys.to_vec();
    run_tmux("send-keys", move || {
        let mut cmd = SendKeys::new().target_pane(target.as_str());
        for k in &keys {
            cmd = cmd.key(k.as_str());
        }
        exec_tmux("send-keys", &target, cmd)?;
        Ok(())
    })
    .await
}
