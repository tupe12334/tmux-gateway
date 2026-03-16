use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::preconditions::require_session_exists;
use crate::validation::SessionName;

use super::TmuxError;

/// Pure: build the tmux command specification for renaming a session.
pub fn build_rename_session_command(
    target: &SessionName,
    new_name: &SessionName,
) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "rename-session".into(),
        "-t".into(),
        target.as_str().into(),
        new_name.as_str().into(),
    ])
}

/// Rename a session.
///
/// [tmux docs](https://man.openbsd.org/tmux#rename-session)
#[tracing::instrument(skip(executor))]
pub async fn rename_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
    new_name: &SessionName,
) -> Result<(), TmuxError> {
    require_session_exists(executor, target).await?;
    let spec = build_rename_session_command(target, new_name);
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
