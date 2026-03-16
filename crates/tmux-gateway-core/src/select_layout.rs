use crate::command_spec::TmuxCommandSpec;
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

/// Pure: build the tmux command specification for selecting a layout.
pub fn build_select_layout_command(target: &WindowTarget, layout: &PaneLayout) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "select-layout".into(),
        "-t".into(),
        target.as_str().into(),
        layout.as_tmux_arg().into(),
    ])
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
    let spec = build_select_layout_command(target, &layout);
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
