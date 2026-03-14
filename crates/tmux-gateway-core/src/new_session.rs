use crate::TmuxSession;
use crate::executor::{exec_tmux, run_tmux};
use crate::sessions::parse_session_line;
use crate::validation::validate_session_name;
use tmux_interface::NewSession;

use super::TmuxError;

pub async fn new_session(name: &str) -> Result<TmuxSession, TmuxError> {
    validate_session_name(name)?;
    let name = name.to_string();
    run_tmux("new-session", move || {
        let output = exec_tmux(
            "new-session",
            &name,
            NewSession::new()
                .detached()
                .session_name(name.as_str())
                .print()
                .format(
                    "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
                ),
        )?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_session_line(stdout.trim())
    })
    .await
}
