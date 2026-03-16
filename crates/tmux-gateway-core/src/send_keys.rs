use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::validation::{PaneTarget, ValidationError};

use super::TmuxError;

/// Pure: build the tmux command specification for sending keys.
pub fn build_send_keys_command(target: &PaneTarget, keys: &[String]) -> TmuxCommandSpec {
    let mut args = vec![
        "send-keys".to_string(),
        "-t".to_string(),
        target.as_str().to_string(),
    ];
    for k in keys {
        args.push(k.clone());
    }
    TmuxCommandSpec::new(args)
}

#[tracing::instrument(skip(executor))]
pub async fn send_keys(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    keys: &[String],
) -> Result<(), TmuxError> {
    send_keys_with_log(executor, target, keys, &NoopLog).await
}

/// Imperative shell: send keys to a pane with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn send_keys_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    keys: &[String],
    log: &dyn LogPort,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    log.log_with_target(
        LogLevel::Info,
        "send-keys",
        target_str,
        &format!("sending {} key(s) to '{target_str}'", keys.len()),
    );
    if keys.is_empty() {
        let e = ValidationError::EmptyInput { field: "keys" };
        log.log_with_target(
            LogLevel::Warn,
            "send-keys",
            target_str,
            &format!("validation rejected — {e}"),
        );
        return Err(e.into());
    }
    let spec = build_send_keys_command(target, keys);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        let err = TmuxError::from_stderr(spec.command_name(), &output.stderr, target_str);
        log.log_with_target(
            LogLevel::Error,
            "send-keys",
            target_str,
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    log.log_with_target(
        LogLevel::Info,
        "send-keys",
        target_str,
        &format!("keys sent successfully to '{target_str}'"),
    );
    Ok(())
}
