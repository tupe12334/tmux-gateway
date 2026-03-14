use crate::executor::TmuxExecutor;
use crate::validation::{validate_session_name, validate_session_target};

use super::TmuxError;

pub async fn rename_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    new_name: &str,
) -> Result<(), TmuxError> {
    validate_session_target(target)?;
    validate_session_name(new_name)?;
    let output = executor
        .execute(&["rename-session", "-t", target, new_name])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "rename-session",
            &output.stderr,
            target,
        ));
    }
    Ok(())
}
