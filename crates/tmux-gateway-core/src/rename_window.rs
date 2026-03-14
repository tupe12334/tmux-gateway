use crate::executor::TmuxExecutor;
use crate::validation::{validate_window_name, validate_window_target};

use super::TmuxError;

pub async fn rename_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    new_name: &str,
) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    validate_window_name(new_name)?;
    let output = executor
        .execute(&["rename-window", "-t", target, new_name])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "rename-window",
            &output.stderr,
            target,
        ));
    }
    Ok(())
}
