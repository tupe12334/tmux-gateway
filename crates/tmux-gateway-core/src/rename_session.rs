use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
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

/// Imperative shell: orchestrate command building and I/O.
#[tracing::instrument(skip(executor))]
pub async fn rename_session(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &SessionName,
    new_name: &SessionName,
) -> Result<(), TmuxError> {
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
