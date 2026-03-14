use crate::validation::validate_session_name;
use tmux_interface::{NewSession, Tmux};

use super::TmuxError;
use crate::timeout::spawn_blocking_with_timeout;

pub async fn new_session(name: &str) -> Result<String, TmuxError> {
    validate_session_name(name)?;
    let name = name.to_string();
    spawn_blocking_with_timeout("new-session", move || {
        let output = Tmux::with_command(NewSession::new().detached().session_name(name.as_str()))
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "new-session".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("new-session", &stderr, &name));
        }

        Ok(name)
    })
    .await
}
