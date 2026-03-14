use crate::executor::TmuxExecutor;
use crate::validation::validate_pane_target;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn kill_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let output = executor.execute(&["kill-pane", "-t", target]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr("kill-pane", &output.stderr, target));
    }
    Ok(())
}
