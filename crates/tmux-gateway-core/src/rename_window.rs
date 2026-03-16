use crate::executor::TmuxExecutor;
use crate::preconditions::require_window_exists;
use crate::validation::{WindowTarget, validate_window_name};

use super::TmuxError;

/// Rename a window.
///
/// [tmux docs](https://man.openbsd.org/tmux#rename-window)
#[tracing::instrument(skip(executor))]
pub async fn rename_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
    new_name: &str,
) -> Result<(), TmuxError> {
    validate_window_name(new_name)?;
    require_window_exists(executor, target).await?;
    let target_str = target.as_str();
    let output = executor
        .execute(&["rename-window", "-t", target_str, new_name])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "rename-window",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
