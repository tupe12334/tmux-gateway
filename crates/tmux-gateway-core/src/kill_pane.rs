use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

use super::TmuxError;

/// Pure: build the tmux command specification for killing a pane.
pub fn build_kill_pane_command(target: &PaneTarget) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "kill-pane".into(),
        "-t".into(),
        target.as_str().into(),
    ])
}

/// Imperative shell: orchestrate command building and I/O.
#[tracing::instrument(skip(executor))]
pub async fn kill_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
) -> Result<(), TmuxError> {
    let spec = build_kill_pane_command(target);
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
