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

#[tracing::instrument(skip(executor))]
pub async fn resize_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    direction: ResizeDirection,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    let (flag, amount) = match direction {
        ResizeDirection::Up(n) => ("-U", n),
        ResizeDirection::Down(n) => ("-D", n),
        ResizeDirection::Left(n) => ("-L", n),
        ResizeDirection::Right(n) => ("-R", n),
    };
    let amount_str = amount.to_string();
    let output = executor
        .execute(&["resize-pane", flag, "-t", target_str, &amount_str])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "resize-pane",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
