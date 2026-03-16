use crate::executor::TmuxExecutor;
use crate::validation::WindowTarget;

use super::TmuxError;

/// Select (activate) a window.
///
/// [tmux docs](https://man.openbsd.org/tmux#select-window)
#[tracing::instrument(skip(executor))]
pub async fn select_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    let output = executor
        .execute(&["select-window", "-t", target_str])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "select-window",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
