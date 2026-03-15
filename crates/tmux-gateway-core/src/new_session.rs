use crate::TmuxSession;
use crate::events::{EventSender, TmuxEvent};
use crate::executor::TmuxExecutor;
use crate::sessions::parse_session_line;
use crate::validation::{validate_command, validate_session_name};

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn new_session(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
    command: Option<&str>,
) -> Result<TmuxSession, TmuxError> {
    new_session_with_events(executor, name, command, None).await
}

#[tracing::instrument(skip(executor, event_tx))]
pub async fn new_session_with_events(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
    command: Option<&str>,
    event_tx: Option<&EventSender>,
) -> Result<TmuxSession, TmuxError> {
    validate_session_name(name)?;
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }
    let format_str = "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}";
    let mut args = vec!["new-session", "-d", "-s", name];
    if let Some(cmd) = command {
        args.push(cmd);
    }
    args.extend_from_slice(&["-P", "-F", format_str]);
    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr("new-session", &output.stderr, name));
    }
    let session = parse_session_line(output.stdout.trim())?;

    if let Some(tx) = event_tx {
        let _ = tx.send(TmuxEvent::SessionCreated {
            name: session.name.clone(),
        });
    }

    Ok(session)
}
