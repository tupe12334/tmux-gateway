use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::WindowTarget;

use super::TmuxError;

/// Pure: build the tmux command specification for selecting a window.
pub fn build_select_window_command(target: &WindowTarget) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "select-window".into(),
        "-t".into(),
        target.as_str().into(),
    ])
}

/// Select (activate) a window.
///
/// [tmux docs](https://man.openbsd.org/tmux#select-window)
#[tracing::instrument(skip(executor))]
pub async fn select_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
) -> Result<(), TmuxError> {
    let spec = build_select_window_command(target);
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
