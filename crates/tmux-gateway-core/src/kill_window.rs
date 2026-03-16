use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::validation::WindowTarget;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn kill_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
) -> Result<(), TmuxError> {
    kill_window_with_log(executor, target, &NoopLog).await
}

/// Kill a window with domain-level logging.
#[tracing::instrument(skip(executor, log))]
pub async fn kill_window_with_log(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
    log: &dyn LogPort,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    log.log_with_target(
        LogLevel::Info,
        "kill-window",
        target_str,
        &format!("killing window '{target_str}'"),
    );
    let output = executor.execute(&["kill-window", "-t", target_str]).await?;
    if !output.success {
        let err = TmuxError::from_stderr("kill-window", &output.stderr, target_str);
        log.log_with_target(
            LogLevel::Error,
            "kill-window",
            target_str,
            &format!("tmux command failed: {err}"),
        );
        return Err(err);
    }
    log.log_with_target(
        LogLevel::Info,
        "kill-window",
        target_str,
        &format!("window '{target_str}' killed successfully"),
    );
    Ok(())
}
