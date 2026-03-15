use crate::TmuxPane;
use crate::executor::TmuxExecutor;
use crate::list_panes::parse_pane_line;
use crate::validation::{validate_command, validate_pane_target};

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn split_window(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    horizontal: bool,
    command: Option<&str>,
) -> Result<TmuxPane, TmuxError> {
    validate_pane_target(target)?;
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }
    let direction = if horizontal { "-h" } else { "-v" };
    let format_str = "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}\t#{pane_pid}";
    let mut args = vec!["split-window", "-d", direction, "-t", target];
    if let Some(cmd) = command {
        args.push(cmd);
    }
    args.extend_from_slice(&["-P", "-F", format_str]);
    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "split-window",
            &output.stderr,
            target,
        ));
    }
    parse_pane_line(output.stdout.trim())
}
