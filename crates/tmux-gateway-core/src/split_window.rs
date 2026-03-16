use crate::TmuxPane;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::list_panes::parse_pane_line;
use crate::validation::{PaneTarget, validate_command};

use super::TmuxError;

/// Pure: build the tmux command specification for splitting a window.
pub fn build_split_window_command(
    target: &PaneTarget,
    horizontal: bool,
    command: Option<&str>,
) -> TmuxCommandSpec {
    let direction = if horizontal { "-h" } else { "-v" };
    let format_str = "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}\t#{pane_pid}";
    let mut args = vec![
        "split-window".to_string(),
        "-d".to_string(),
        direction.to_string(),
        "-t".to_string(),
        target.as_str().to_string(),
    ];
    if let Some(cmd) = command {
        args.push(cmd.to_string());
    }
    args.extend_from_slice(&["-P".to_string(), "-F".to_string(), format_str.to_string()]);
    TmuxCommandSpec::new(args)
}

/// Imperative shell: orchestrate validation, command building, I/O, and parsing.
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
    let spec = build_split_window_command(target, horizontal, command);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            target.as_str(),
        ));
    }
    parse_pane_line(output.stdout.trim())
}
