use crate::TmuxWindow;
use crate::executor::{exec_tmux, run_tmux};
use crate::list_windows::parse_window_line;
use crate::validation::{validate_session_target, validate_window_name};
use tmux_interface::NewWindow;

use super::TmuxError;

pub async fn new_window(session: &str, name: &str) -> Result<TmuxWindow, TmuxError> {
    validate_session_target(session)?;
    validate_window_name(name)?;
    let session = session.to_string();
    let name = name.to_string();
    run_tmux("new-window", move || {
        let output = exec_tmux(
            "new-window",
            &session,
            NewWindow::new()
                .detached()
                .target_window(session.as_str())
                .window_name(name.as_str())
                .print()
                .format("#{window_id}\t#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}"),
        )?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_window_line(stdout.trim())
    })
    .await
}
