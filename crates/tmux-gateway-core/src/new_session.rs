use crate::TmuxSession;
use crate::command_spec::TmuxCommandSpec;
use crate::events::{EventSender, TmuxEvent};
use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::preconditions::require_session_not_exists;
use crate::sessions::parse_session_line;
use crate::validation::{
    SessionName, validate_command, validate_command_arg, validate_working_directory,
};

use super::TmuxError;

/// Pure: build the tmux command specification for creating a new session.
pub fn build_new_session_command(
    name: &SessionName,
    command: Option<&str>,
    working_directory: Option<&str>,
) -> TmuxCommandSpec {
    let format_str = "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}";
    let mut args = vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        name.as_str().to_string(),
    ];
    if let Some(dir) = working_directory {
        args.push("-c".to_string());
        args.push(dir.to_string());
    }
    if let Some(cmd) = command {
        args.push(cmd.to_string());
    }
    args.extend_from_slice(&["-P".to_string(), "-F".to_string(), format_str.to_string()]);
    TmuxCommandSpec::new(args)
}

/// Pure: build the tmux command specification for creating a new session
/// with a multi-arg command (direct exec, no shell interpretation).
pub fn build_new_session_command_with_args(
    name: &SessionName,
    command_args: &[&str],
    working_directory: Option<&str>,
) -> TmuxCommandSpec {
    let format_str = "#{session_id}\t#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}";
    let mut args = vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        name.as_str().to_string(),
    ];
    if let Some(dir) = working_directory {
        args.push("-c".to_string());
        args.push(dir.to_string());
    }
    args.extend_from_slice(&["-P".to_string(), "-F".to_string(), format_str.to_string()]);
    for arg in command_args {
        args.push(arg.to_string());
    }
    TmuxCommandSpec::new(args)
}

#[tracing::instrument(skip(executor))]
pub async fn new_session(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    working_directory: Option<&str>,
) -> Result<TmuxSession, TmuxError> {
    new_session_with_log(executor, name, command, working_directory, &NoopLog).await
}

#[tracing::instrument(skip(executor, log))]
pub async fn new_session_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    working_directory: Option<&str>,
    log: &dyn LogPort,
) -> Result<TmuxSession, TmuxError> {
    new_session_inner(executor, name, command, working_directory, None, log).await
}

#[tracing::instrument(skip(executor, event_tx))]
pub async fn new_session_with_events(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    working_directory: Option<&str>,
    event_tx: Option<&EventSender>,
) -> Result<TmuxSession, TmuxError> {
    new_session_inner(
        executor,
        name,
        command,
        working_directory,
        event_tx,
        &NoopLog,
    )
    .await
}

async fn new_session_inner(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command: Option<&str>,
    working_directory: Option<&str>,
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
    if let Some(dir) = working_directory
        && let Err(e) = validate_working_directory(dir)
    {
        log.log(
            LogLevel::Warn,
            "new-session",
            &format!("validation rejected working directory — {e}"),
        );
        return Err(e.into());
    }
    let spec = build_new_session_command(name, command, working_directory);
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

#[tracing::instrument(skip(executor))]
pub async fn new_session_with_args(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command_args: &[&str],
    working_directory: Option<&str>,
) -> Result<TmuxSession, TmuxError> {
    new_session_with_args_inner(
        executor,
        name,
        command_args,
        working_directory,
        None,
        &NoopLog,
    )
    .await
}

#[tracing::instrument(skip(executor, event_tx))]
pub async fn new_session_with_args_and_events(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command_args: &[&str],
    working_directory: Option<&str>,
    event_tx: Option<&EventSender>,
) -> Result<TmuxSession, TmuxError> {
    new_session_with_args_inner(
        executor,
        name,
        command_args,
        working_directory,
        event_tx,
        &NoopLog,
    )
    .await
}

async fn new_session_with_args_inner(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    command_args: &[&str],
    working_directory: Option<&str>,
    event_tx: Option<&EventSender>,
    log: &dyn LogPort,
) -> Result<TmuxSession, TmuxError> {
    require_session_not_exists(executor, name).await?;
    let name_str = name.as_str();
    log.log(
        LogLevel::Info,
        "new-session",
        &format!("creating session '{name_str}' with args"),
    );
    for arg in command_args {
        if let Err(e) = validate_command_arg(arg) {
            log.log(
                LogLevel::Warn,
                "new-session",
                &format!("validation rejected command arg — {e}"),
            );
            return Err(e.into());
        }
    }
    if let Some(dir) = working_directory
        && let Err(e) = validate_working_directory(dir)
    {
        log.log(
            LogLevel::Warn,
            "new-session",
            &format!("validation rejected working directory — {e}"),
        );
        return Err(e.into());
    }
    let spec = build_new_session_command_with_args(name, command_args, working_directory);
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
