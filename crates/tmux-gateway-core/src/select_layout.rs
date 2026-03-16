use crate::executor::TmuxExecutor;
use crate::validation::WindowTarget;

use super::TmuxError;

/// Layout presets supported by tmux `select-layout`.
///
/// [tmux docs](https://man.openbsd.org/tmux#select-layout)
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

/// Apply a layout to a window.
///
/// [tmux docs](https://man.openbsd.org/tmux#select-layout)
#[tracing::instrument(skip(executor))]
pub async fn select_layout(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &WindowTarget,
    layout: PaneLayout,
) -> Result<(), TmuxError> {
    let target_str = target.as_str();
    let output = executor
        .execute(&["select-layout", "-t", target_str, layout.as_tmux_arg()])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "select-layout",
            &output.stderr,
            target_str,
        ));
    }
    Ok(())
}
