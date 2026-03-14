use crate::executor::{exec_tmux, run_tmux};
use crate::validation::{validate_session_name, validate_session_target};
use tmux_interface::RenameSession as TmuxRenameSession;

use super::TmuxError;

pub async fn rename_session(target: &str, new_name: &str) -> Result<(), TmuxError> {
    validate_session_target(target)?;
    validate_session_name(new_name)?;
    let target = target.to_string();
    let new_name = new_name.to_string();
    run_tmux("rename-session", move || {
        exec_tmux(
            "rename-session",
            &target,
            TmuxRenameSession::new()
                .target_session(target.as_str())
                .new_name(new_name.as_str()),
        )?;
        Ok(())
    })
    .await
}
