use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::validation::validate_window_target;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn kill_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<(), TmuxError> {
    kill_window_with_log(executor, target, &NoopLog).await
}

/// Kill a window with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn kill_window_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    log: &dyn LogPort,
) -> Result<(), TmuxError> {
    log.log_with_target(
        LogLevel::Info,
        "kill-window",
        target,
        &format!("killing window '{target}'"),
    );
    if let Err(e) = validate_window_target(target) {
        log.log_with_target(
            LogLevel::Warn,
            "kill-window",
            target,
            &format!("validation rejected target '{target}' — {e}"),
        );
        return Err(e.into());
    }
    let output = executor.execute(&["kill-window", "-t", target]).await?;
    if !output.success {
        let err = TmuxError::from_stderr("kill-window", &output.stderr, target);
        log.log_with_target(
            LogLevel::Error,
            "kill-window",
            target,
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    log.log_with_target(
        LogLevel::Info,
        "kill-window",
        target,
        &format!("window '{target}' killed successfully"),
    );
    Ok(())
}
