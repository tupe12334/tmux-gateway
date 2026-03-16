use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::log_port::{LogLevel, LogPort, NoopLog};
use crate::validation::WindowTarget;

use super::TmuxError;

/// Pure: build the tmux command specification for killing a window.
pub fn build_kill_window_command(target: &WindowTarget) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "kill-window".into(),
        "-t".into(),
        target.as_str().into(),
    ])
}

#[tracing::instrument(skip(executor))]
pub async fn kill_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
) -> Result<(), TmuxError> {
    kill_window_with_log(executor, target, &NoopLog).await
}

/// Imperative shell: kill a window with domain-level logging.
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
    let spec = build_kill_window_command(target);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        let err = TmuxError::from_stderr(spec.command_name(), &output.stderr, target_str);
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
