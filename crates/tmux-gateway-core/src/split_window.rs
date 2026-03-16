use crate::TmuxPane;
use crate::executor::TmuxExecutor;
use crate::list_panes::parse_pane_line;
use crate::validation::{PaneTarget, validate_command};

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn split_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    horizontal: bool,
    command: Option<&str>,
) -> Result<TmuxPane, TmuxError> {
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }
    let target_str = target.as_str();
    let direction = if horizontal { "-h" } else { "-v" };
    let format_str = "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}\t#{pane_pid}";
    let mut args = vec!["split-window", "-d", direction, "-t", target_str];
    if let Some(cmd) = command {
        args.push(cmd);
    }
    args.extend_from_slice(&["-P", "-F", format_str]);
    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "split-window",
            &output.stderr,
            target_str,
        ));
    }
    parse_pane_line(output.stdout.trim())
}
