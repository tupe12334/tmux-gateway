use crate::TmuxPane;
use crate::executor::{exec_tmux, run_tmux};
use crate::list_panes::parse_pane_line;
use crate::validation::validate_pane_target;
use tmux_interface::SplitWindow;

use super::TmuxError;

pub async fn split_window(target: &str, horizontal: bool) -> Result<TmuxPane, TmuxError> {
    validate_pane_target(target)?;
    let target = target.to_string();
    run_tmux("split-window", move || {
        let mut cmd = SplitWindow::new()
            .detached()
            .target_pane(target.as_str())
            .print()
            .format("#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}");
        if horizontal {
            cmd = cmd.horizontal();
        } else {
            cmd = cmd.vertical();
        }
        let output = exec_tmux("split-window", &target, cmd)?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_pane_line(stdout.trim())
    })
    .await
}
