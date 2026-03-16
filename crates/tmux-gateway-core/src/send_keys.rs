use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::validation::{ValidationError, validate_pane_target};

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn send_keys(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    keys: &[String],
) -> Result<(), TmuxError> {
    send_keys_with_log(executor, target, keys, &NoopLog).await
}

/// Send keys to a pane with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn send_keys_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    keys: &[String],
    log: &dyn LogPort,
) -> Result<(), TmuxError> {
    log.log_with_target(
        LogLevel::Info,
        "send-keys",
        target,
        &format!("sending {} key(s) to '{target}'", keys.len()),
    );
    if let Err(e) = validate_pane_target(target) {
        log.log_with_target(
            LogLevel::Warn,
            "send-keys",
            target,
            &format!("validation rejected target '{target}' — {e}"),
        );
        return Err(e.into());
    }
    if keys.is_empty() {
        let e = ValidationError::EmptyInput { field: "keys" };
        log.log_with_target(
            LogLevel::Warn,
            "send-keys",
            target,
            &format!("validation rejected — {e}"),
        );
        return Err(e.into());
    }
    let mut args: Vec<&str> = vec!["send-keys", "-t", target];
    for k in keys {
        args.push(k.as_str());
    }
    let output = executor.execute(&args).await?;
    if !output.success {
        let err = TmuxError::from_stderr("send-keys", &output.stderr, target);
        log.log_with_target(
            LogLevel::Error,
            "send-keys",
            target,
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    log.log_with_target(
        LogLevel::Info,
        "send-keys",
        target,
        &format!("keys sent successfully to '{target}'"),
    );
    Ok(())
}
