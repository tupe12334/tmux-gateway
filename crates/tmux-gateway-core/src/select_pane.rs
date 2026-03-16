use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn select_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    let output = executor.execute(&["select-pane", "-t", target_str]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "select-pane",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
