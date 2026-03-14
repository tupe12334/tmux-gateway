use crate::executor::TmuxExecutor;
use crate::validation::validate_window_target;

use super::TmuxError;

pub async fn kill_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    let output = executor.execute(&["kill-window", "-t", target]).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "kill-window",
            &output.stderr,
            target,
        ));
    }
    Ok(())
}
