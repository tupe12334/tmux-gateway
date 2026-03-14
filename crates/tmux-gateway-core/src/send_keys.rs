use crate::executor::TmuxExecutor;
use crate::validation::{ValidationError, validate_pane_target};

use super::TmuxError;

pub async fn send_keys(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    keys: &[String],
) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    if keys.is_empty() {
        return Err(ValidationError::EmptyInput { field: "keys" }.into());
    }
    let mut args: Vec<&str> = vec!["send-keys", "-t", target];
    for k in keys {
        args.push(k.as_str());
    }
    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr("send-keys", &output.stderr, target));
    }
    Ok(())
}
