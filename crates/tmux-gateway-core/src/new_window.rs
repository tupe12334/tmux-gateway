use crate::TmuxWindow;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::list_windows::parse_window_line;
use crate::validation::{SessionName, validate_command, validate_window_name};

use super::TmuxError;

/// Pure: build the tmux command specification for creating a new window.
pub fn build_new_window_command(
    session: &SessionName,
    name: &str,
    command: Option<&str>,
) -> TmuxCommandSpec {
    let format_str =
        "#{window_id}\t#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}";
    let mut args = vec![
        "new-window".to_string(),
        "-d".to_string(),
        "-t".to_string(),
        session.as_str().to_string(),
        "-n".to_string(),
        name.to_string(),
    ];
    if let Some(cmd) = command {
        args.push(cmd.to_string());
    }
    args.extend_from_slice(&["-P".to_string(), "-F".to_string(), format_str.to_string()]);
    TmuxCommandSpec::new(args)
}

/// Create a new window in a session.
///
/// [tmux docs](https://man.openbsd.org/tmux#new-window)
#[tracing::instrument(skip(executor))]
pub async fn new_window(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &SessionName,
    name: &str,
    command: Option<&str>,
) -> Result<TmuxWindow, TmuxError> {
    validate_window_name(name)?;
    if let Some(cmd) = command {
        validate_command(cmd)?;
    }
    let spec = build_new_window_command(session, name, command);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            session.as_str(),
        ));
    }
    parse_window_line(output.stdout.trim())
}
