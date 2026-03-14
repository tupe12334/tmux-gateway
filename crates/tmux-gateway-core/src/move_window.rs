use tmux_interface::MoveWindow;

use super::TmuxError;
use crate::executor::{exec_tmux, run_tmux};
use crate::validation::{validate_session_target, validate_window_target};

/// Move a window from one session to another.
///
/// `source` is in window target format (`session:window`).
/// `destination_session` is a session name.
pub async fn move_window(source: &str, destination_session: &str) -> Result<(), TmuxError> {
    validate_window_target(source)?;
    validate_session_target(destination_session)?;
    let source = source.to_string();
    let destination_session = destination_session.to_string();
    run_tmux("move-window", move || {
        exec_tmux(
            "move-window",
            &source,
            MoveWindow::new()
                .src_window(source.as_str())
                .dst_window(destination_session.as_str()),
        )?;
        Ok(())
    })
    .await
}
