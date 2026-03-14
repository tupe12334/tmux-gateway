use super::TmuxError;
use crate::executor::TmuxExecutor;
use crate::validation::{validate_session_target, validate_window_target};

/// Move a window from one session to another.
///
/// `source` is in window target format (`session:window`).
/// `destination_session` is a session name.
pub async fn move_window(
    executor: &(impl TmuxExecutor + ?Sized),
    source: &str,
    destination_session: &str,
) -> Result<(), TmuxError> {
    validate_window_target(source)?;
    validate_session_target(destination_session)?;
    let output = executor
        .execute(&["move-window", "-s", source, "-t", destination_session])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "move-window",
            &output.stderr,
            source,
        ));
    }
    Ok(())
}
