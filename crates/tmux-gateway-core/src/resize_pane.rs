use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::PaneTarget;

use super::TmuxError;

/// Direction (and amount) by which to resize a pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeDirection {
    Up(u32),
    Down(u32),
    Left(u32),
    Right(u32),
}

/// Pure: build the tmux command specification for resizing a pane.
pub fn build_resize_pane_command(
    target: &PaneTarget,
    direction: ResizeDirection,
) -> TmuxCommandSpec {
    let (flag, amount) = match direction {
        ResizeDirection::Up(n) => ("-U", n),
        ResizeDirection::Down(n) => ("-D", n),
        ResizeDirection::Left(n) => ("-L", n),
        ResizeDirection::Right(n) => ("-R", n),
    };
    TmuxCommandSpec::new(vec![
        "resize-pane".into(),
        flag.into(),
        "-t".into(),
        target.as_str().into(),
        amount.to_string(),
    ])
}

/// Imperative shell: orchestrate command building and I/O.
#[tracing::instrument(skip(executor))]
pub async fn resize_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    direction: ResizeDirection,
) -> Result<(), TmuxError> {
    let spec = build_resize_pane_command(target, direction);
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
