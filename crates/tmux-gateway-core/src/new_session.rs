use crate::TmuxSession;
use crate::events::{EventSender, TmuxEvent};
use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::preconditions::require_session_not_exists;
use crate::sessions::parse_session_line;
use crate::validation::{SessionName, validate_command};

use super::TmuxError;

/// Create a new tmux session.
///
/// [tmux docs](https://man.openbsd.org/tmux#new-session)
#[tracing::instrument(skip(executor))]
pub async fn new_session(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
) -> Result<TmuxSession, TmuxError> {
    new_session_with_log(executor, name, command, &NoopLog).await
}

/// Create a new session with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn new_session_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    log: &dyn LogPort,
) -> Result<TmuxSession, TmuxError> {
    new_session_inner(executor, name, command, None, log).await
}

#[tracing::instrument(skip(executor, event_tx))]
pub async fn new_session_with_events(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    event_tx: Option<&EventSender>,
) -> Result<TmuxSession, TmuxError> {
    new_session_inner(executor, name, command, event_tx, &NoopLog).await
}

async fn new_session_inner(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    event_tx: Option<&EventSender>,
    log: &dyn LogPort,
) -> Result<TmuxSession, TmuxError> {
    require_session_not_exists(executor, name).await?;
    let name_str = name.as_str();
    log.log(
        LogLevel::Info,
        "new-session",
        &format!("creating session '{name_str}'"),
    );
    if let Some(cmd) = command
        && let Err(e) = validate_command(cmd)
    {
        log.log(
            LogLevel::Warn,
            "new-session",
            &format!("validation rejected command — {e}"),
        );
        return Err(e.into());
    }
    let format_str = "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}";
    let mut args = vec!["new-session", "-d", "-s", name_str];
    if let Some(cmd) = command {
        args.push(cmd);
    }
    args.extend_from_slice(&["-P", "-F", format_str]);
    let output = executor.execute(&args).await?;
    if !output.success {
        let err = TmuxError::from_stderr("new-session", &output.stderr, name_str);
        log.log(
            LogLevel::Error,
            "new-session",
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    let session = parse_session_line(output.stdout.trim())?;

    if let Some(tx) = event_tx {
        let _ = tx.send(TmuxEvent::SessionCreated {
            name: session.name.clone(),
        });
    }

    log.log(
        LogLevel::Info,
        "new-session",
        &format!("session '{}' created successfully", session.name),
    );
    Ok(session)
}
