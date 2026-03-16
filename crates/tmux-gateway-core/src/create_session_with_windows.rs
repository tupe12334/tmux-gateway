use super::TmuxError;
use super::sessions::TmuxSession;
use crate::executor::TmuxExecutor;
use crate::validation::{SessionName, WindowTarget, validate_window_name};

/// Create a session with pre-configured windows in a single domain operation.
///
/// Creates the session first, then adds each named window. The initial default
/// window created by tmux is renamed to the first window name if provided.
/// On partial failure (session created but a window fails), the session is
/// killed to maintain atomicity.
#[tracing::instrument(skip(executor))]
pub async fn create_session_with_windows(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
    window_names: &[String],
) -> Result<TmuxSession, TmuxError> {
    for wn in window_names {
        validate_window_name(wn)?;
    }

    // Step 1: Create the session
    super::new_session(executor, name, None).await?;

    // Step 2: If window names provided, rename the default window and create additional ones
    if let Some((first, rest)) = window_names.split_first() {
        // Rename the default window (index 0) to the first requested name
        let default_window_target = WindowTarget::try_from(format!("{}:0", name).as_str())?;
        if let Err(e) = super::rename_window(executor, &default_window_target, first).await {
            // Rollback: kill the session
            let _ = super::kill_session(executor, name).await;
            return Err(e);
        }

        // Create remaining windows
        for wn in rest {
            if let Err(e) = super::new_window(executor, name, wn, None).await {
                // Rollback: kill the session (cleans up all windows)
                let _ = super::kill_session(executor, name).await;
                return Err(e);
            }
        }
    }

    // Step 3: Return the created session info
    super::get_session(executor, name.as_str())
        .await?
        .ok_or_else(|| TmuxError::SessionNotFound(name.as_str().to_string()))
}
