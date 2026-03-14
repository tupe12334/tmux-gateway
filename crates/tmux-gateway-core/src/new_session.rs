use crate::TmuxSession;
use crate::events::{EventSender, TmuxEvent};
use crate::executor::{exec_tmux, run_tmux};
use crate::sessions::parse_session_line;
use crate::validation::validate_session_name;
use tmux_interface::NewSession;

use super::TmuxError;

pub async fn new_session(name: &str) -> Result<TmuxSession, TmuxError> {
    new_session_with_events(name, None).await
}

pub async fn new_session_with_events(
    name: &str,
    event_tx: Option<&EventSender>,
) -> Result<TmuxSession, TmuxError> {
    validate_session_name(name)?;
    let name = name.to_string();
    let session = run_tmux("new-session", {
        let name = name.clone();
        move || {
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
        }
    })
    .await?;

    if let Some(tx) = event_tx {
        let _ = tx.send(TmuxEvent::SessionCreated {
            name: session.name.clone(),
        });
    }

    Ok(session)
}
