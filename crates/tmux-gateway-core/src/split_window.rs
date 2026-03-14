use crate::TmuxPane;
use crate::executor::TmuxExecutor;
use crate::list_panes::parse_pane_line;
use crate::validation::validate_pane_target;

use super::TmuxError;

pub async fn split_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    horizontal: bool,
) -> Result<TmuxPane, TmuxError> {
    validate_pane_target(target)?;
    let direction = if horizontal { "-h" } else { "-v" };
    let output = executor
        .execute(&[
            "split-window",
            "-d",
            direction,
            "-t",
            target,
            "-P",
            "-F",
            "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}",
        ])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "split-window",
            &output.stderr,
            target,
        ));
    }
    parse_pane_line(output.stdout.trim())
}
