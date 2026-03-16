use super::TmuxError;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::{SessionName, WindowTarget};

/// Pure: build the tmux command specification for moving a window.
pub fn build_move_window_command(
    source: &WindowTarget,
    destination_session: &SessionName,
) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "move-window".into(),
        "-s".into(),
        source.as_str().into(),
        "-t".into(),
        destination_session.as_str().into(),
    ])
}

/// Move a window from one session to another.
///
/// `source` is in window target format (`session:window`).
/// `destination_session` is a session name.
///
/// [tmux docs](https://man.openbsd.org/tmux#move-window)
#[tracing::instrument(skip(executor))]
pub async fn move_window(
    executor: &(impl TmuxExecutor + ?Sized),
    source: &WindowTarget,
    destination_session: &SessionName,
) -> Result<(), TmuxError> {
    let spec = build_move_window_command(source, destination_session);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            source.as_str(),
        ));
    }
    Ok(())
}
