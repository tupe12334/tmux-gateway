use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::validate_pane_target;

/// Swap two panes by their targets (format: `session:window.pane`).
pub async fn swap_panes(
    executor: &(impl TmuxExecutor + ?Sized),
    src: &str,
    dst: &str,
) -> Result<(), TmuxError> {
    validate_pane_target(src)?;
    validate_pane_target(dst)?;
    let output = executor
        .execute(&["swap-pane", "-s", src, "-t", dst])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr("swap-pane", &output.stderr, src));
    }
    Ok(())
}
