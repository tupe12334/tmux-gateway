use crate::executor::TmuxExecutor;
use crate::validation::validate_pane_target;

use super::TmuxError;

pub async fn select_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let output = executor.execute(&["select-pane", "-t", target]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "select-pane",
            &output.stderr,
            target,
        ));
    }
    Ok(())
}
