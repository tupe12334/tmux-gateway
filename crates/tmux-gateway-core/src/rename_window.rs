use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::preconditions::require_window_exists;
use crate::validation::{WindowTarget, validate_window_name};

use super::TmuxError;

/// Pure: build the tmux command specification for renaming a window.
pub fn build_rename_window_command(target: &WindowTarget, new_name: &str) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "rename-window".into(),
        "-t".into(),
        target.as_str().into(),
        new_name.into(),
    ])
}

/// Rename a window.
///
/// [tmux docs](https://man.openbsd.org/tmux#rename-window)
#[tracing::instrument(skip(executor))]
pub async fn rename_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
    new_name: &str,
) -> Result<(), TmuxError> {
    validate_window_name(new_name)?;
    require_window_exists(executor, target).await?;
    let spec = build_rename_window_command(target, new_name);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            target.as_str(),
        ));
    }
    Ok(())
}
