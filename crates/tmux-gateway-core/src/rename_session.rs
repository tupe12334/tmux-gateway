use crate::executor::TmuxExecutor;
use crate::preconditions::require_session_exists;
use crate::validation::SessionName;

use super::TmuxError;

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
