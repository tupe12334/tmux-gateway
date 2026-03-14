use crate::executor::TmuxExecutor;
use crate::validation::validate_pane_target;

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
    target: &str,
    direction: ResizeDirection,
) -> Result<(), TmuxError> {
    validate_pane_target(target)?;
    let (flag, amount) = match direction {
        ResizeDirection::Up(n) => ("-U", n),
        ResizeDirection::Down(n) => ("-D", n),
        ResizeDirection::Left(n) => ("-L", n),
        ResizeDirection::Right(n) => ("-R", n),
    };
    let amount_str = amount.to_string();
    let output = executor
        .execute(&["resize-pane", flag, "-t", target, &amount_str])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "resize-pane",
            &output.stderr,
            target,
        ));
    }
    Ok(())
}
