use crate::executor::{exec_tmux, run_tmux};
use crate::validation::{validate_window_name, validate_window_target};
use tmux_interface::RenameWindow as TmuxRenameWindow;

use super::TmuxError;

pub async fn rename_window(target: &str, new_name: &str) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    validate_window_name(new_name)?;
    let target = target.to_string();
    let new_name = new_name.to_string();
    run_tmux("rename-window", move || {
        exec_tmux(
            "rename-window",
            &target,
            TmuxRenameWindow::new()
                .target_window(target.as_str())
                .new_name(new_name.as_str()),
        )?;
        Ok(())
    })
    .await
}
