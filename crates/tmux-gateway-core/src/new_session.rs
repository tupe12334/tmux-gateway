use crate::TmuxSession;
use crate::command_spec::TmuxCommandSpec;
use crate::events::{EventSender, TmuxEvent};
use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::preconditions::require_session_not_exists;
use crate::sessions::parse_session_line;
use crate::validation::{SessionName, validate_command};

use super::TmuxError;

/// Pure: build the tmux command specification for creating a new session.
pub fn build_new_session_command(name: &SessionName, command: Option<&str>) -> TmuxCommandSpec {
    let format_str = "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}";
    let mut args = vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        name.as_str().to_string(),
    ];
    if let Some(cmd) = command {
        args.push(cmd.to_string());
    }
    args.extend_from_slice(&["-P".to_string(), "-F".to_string(), format_str.to_string()]);
    TmuxCommandSpec::new(args)
}

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

/// Imperative shell: orchestrate validation, command building, I/O, and parsing.
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
    let spec = build_new_session_command(name, command);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        let err = TmuxError::from_stderr(spec.command_name(), &output.stderr, name_str);
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
