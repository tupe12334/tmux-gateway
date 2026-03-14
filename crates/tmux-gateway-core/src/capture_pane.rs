use crate::executor::TmuxExecutor;

use super::TmuxError;

pub async fn capture_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<String, TmuxError> {
    let output = executor
        .execute(&["capture-pane", "-p", "-t", target])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "capture-pane",
            &output.stderr,
            target,
        ));
    }
    Ok(output.stdout)
}
