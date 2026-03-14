use crate::TmuxSession;
use crate::executor::TmuxExecutor;
use crate::sessions::parse_session_line;
use crate::validation::validate_session_name;

use super::TmuxError;

pub async fn new_session(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<TmuxSession, TmuxError> {
    validate_session_name(name)?;
    let output = executor
        .execute(&[
            "new-session",
            "-d",
            "-s",
            name,
            "-P",
            "-F",
            "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
        ])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr("new-session", &output.stderr, name));
    }
    parse_session_line(output.stdout.trim())
}
