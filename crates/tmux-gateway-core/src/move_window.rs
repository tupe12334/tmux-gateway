use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::{SessionName, WindowTarget};

/// Move a window from one session to another.
///
/// `source` is in window target format (`session:window`).
/// `destination_session` is a session name.
#[tracing::instrument(skip(executor))]
pub async fn move_window(
    executor: &(impl TmuxExecutor + ?Sized),
    source: &WindowTarget,
    destination_session: &SessionName,
) -> Result<(), TmuxError> {
    let source_str = source.as_str();
    let dest_str = destination_session.as_str();
    let output = executor
        .execute(&["move-window", "-s", source_str, "-t", dest_str])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "move-window",
            &output.stderr,
            source_str,
        ));
    }
    Ok(())
}
