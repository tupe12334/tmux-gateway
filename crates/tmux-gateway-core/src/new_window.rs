use crate::TmuxWindow;
use crate::executor::TmuxExecutor;
use crate::list_windows::parse_window_line;
use crate::validation::{validate_command, validate_session_target, validate_window_name};

use super::TmuxError;

#[tracing::instrument(skip(executor))]
pub async fn new_window(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
    name: &str,
    command: Option<&str>,
) -> Result<TmuxWindow, TmuxError> {
    validate_session_target(session)?;
    validate_window_name(name)?;
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }
    let format_str =
        "#{window_id}\t#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}";
    let mut args = vec!["new-window", "-d", "-t", session, "-n", name];
    if let Some(cmd) = command {
        args.push(cmd);
    }
    args.extend_from_slice(&["-P", "-F", format_str]);
    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "new-window",
            &output.stderr,
            session,
        ));
    }
    parse_window_line(output.stdout.trim())
}
