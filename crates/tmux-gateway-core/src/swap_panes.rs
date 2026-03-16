use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

/// Swap two panes by their targets (format: `session:window.pane`).
#[tracing::instrument(skip(executor))]
pub async fn swap_panes(
    executor: &(impl TmuxExecutor + ?Sized),
    src: &PaneTarget,
    dst: &PaneTarget,
) -> Result<(), TmuxError> {
    let src_str = src.as_str();
    let dst_str = dst.as_str();
    let output = executor
        .execute(&["swap-pane", "-s", src_str, "-t", dst_str])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr("swap-pane", &output.stderr, src_str));
    }
    Ok(())
}
