use crate::executor::TmuxExecutor;
use crate::validation::validate_window_target;

use super::TmuxError;

/// Layout presets supported by tmux `select-layout`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneLayout {
    EvenHorizontal,
    EvenVertical,
    MainHorizontal,
    MainVertical,
    Tiled,
    Custom(String),
}

impl PaneLayout {
    /// Returns the tmux layout string passed to `select-layout`.
    fn as_tmux_arg(&self) -> &str {
        match self {
            PaneLayout::EvenHorizontal => "even-horizontal",
            PaneLayout::EvenVertical => "even-vertical",
            PaneLayout::MainHorizontal => "main-horizontal",
            PaneLayout::MainVertical => "main-vertical",
            PaneLayout::Tiled => "tiled",
            PaneLayout::Custom(s) => s,
        }
    }
}

#[tracing::instrument(skip(executor))]
pub async fn select_layout(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    layout: PaneLayout,
) -> Result<(), TmuxError> {
    validate_window_target(target)?;
    let output = executor
        .execute(&["select-layout", "-t", target, layout.as_tmux_arg()])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "select-layout",
            &output.stderr,
            target,
        ));
    }
    Ok(())
}
