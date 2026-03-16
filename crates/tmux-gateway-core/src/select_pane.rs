use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

use super::TmuxError;

/// Pure: build the tmux command specification for selecting a pane.
pub fn build_select_pane_command(target: &PaneTarget) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "select-pane".into(),
        "-t".into(),
        target.as_str().into(),
    ])
}

/// Select (activate) a pane.
///
/// [tmux docs](https://man.openbsd.org/tmux#select-pane)
#[tracing::instrument(skip(executor))]
pub async fn select_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
) -> Result<(), TmuxError> {
    let spec = build_select_pane_command(target);
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
