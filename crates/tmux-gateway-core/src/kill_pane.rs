use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn kill_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    let output = executor.execute(&["kill-pane", "-t", target_str]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "kill-pane",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
