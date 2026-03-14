use crate::TmuxWindow;
use crate::executor::TmuxExecutor;
use crate::list_windows::parse_window_line;
use crate::validation::{validate_session_target, validate_window_name};

use super::TmuxError;

pub async fn new_window(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
    name: &str,
) -> Result<TmuxWindow, TmuxError> {
    validate_session_target(session)?;
    validate_window_name(name)?;
    let output = executor
        .execute(&[
            "new-window",
            "-d",
            "-t",
            session,
            "-n",
            name,
            "-P",
            "-F",
            "#{window_id}\t#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}",
        ])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "new-window",
            &output.stderr,
            session,
        ));
    }
    parse_window_line(output.stdout.trim())
}
