use super::TmuxError;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

/// Pure: build the tmux command specification for swapping panes.
pub fn build_swap_panes_command(src: &PaneTarget, dst: &PaneTarget) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "swap-pane".into(),
        "-s".into(),
        src.as_str().into(),
        "-t".into(),
        dst.as_str().into(),
    ])
}

/// Swap two panes by their targets (format: `session:window.pane`).
#[tracing::instrument(skip(executor))]
pub async fn swap_panes(
    executor: &(impl TmuxExecutor + ?Sized),
    src: &PaneTarget,
    dst: &PaneTarget,
) -> Result<(), TmuxError> {
    let spec = build_swap_panes_command(src, dst);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            src.as_str(),
        ));
    }
    Ok(())
}
