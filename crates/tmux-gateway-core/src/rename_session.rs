use crate::executor::TmuxExecutor;
use crate::validation::SessionName;

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn rename_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
    new_name: &SessionName,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    let new_name_str = new_name.as_str();
    let output = executor
        .execute(&["rename-session", "-t", target_str, new_name_str])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "rename-session",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
