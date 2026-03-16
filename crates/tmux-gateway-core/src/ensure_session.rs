use crate::TmuxSession;
use crate::executor::TmuxExecutor;
use crate::new_session::new_session;
use crate::sessions::get_session;
use crate::validation::SessionName;

use super::TmuxError;

/// Create a session if it doesn't exist, or return the existing one.
///
/// This avoids the TOCTOU race inherent in check-then-create by attempting
/// the create first and falling back to a lookup on `SessionAlreadyExists`.
#[tracing::instrument(skip(executor))]
pub async fn ensure_session(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &SessionName,
) -> Result<TmuxSession, TmuxError> {
    match new_session(executor, name, None).await {
        Ok(session) => Ok(session),
        Err(TmuxError::SessionAlreadyExists(_)) => get_session(executor, name.as_str())
            .await?
            .ok_or_else(|| TmuxError::SessionNotFound(name.as_str().to_string())),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    /// Mock that always succeeds on new-session.
    struct MockCreateSuccess;

    impl TmuxExecutor for MockCreateSuccess {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            if args.first() == Some(&"new-session") {
                Ok(TmuxOutput {
                    stdout: "$0\ttest-sess\t1\t1700000000\t0\n".to_string(),
                    stderr: String::new(),
                    success: true,
                })
            } else {
                panic!("unexpected command: {args:?}");
            }
        }
    }

    #[tokio::test]
    async fn ensure_session_creates_new() {
        let name = SessionName::try_from("test-sess").unwrap();
        let session = ensure_session(&MockCreateSuccess, &name).await.unwrap();
        assert_eq!(session.name, "test-sess");
    }

    /// Mock that fails with duplicate on create, then returns session on list.
    struct MockAlreadyExists;

    impl TmuxExecutor for MockAlreadyExists {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            match args.first() {
                Some(&"new-session") => Ok(TmuxOutput {
                    stdout: String::new(),
                    stderr: "duplicate session: existing".to_string(),
                    success: false,
                }),
                Some(&"list-sessions") => Ok(TmuxOutput {
                    stdout: "$1\texisting\t3\t1700000000\t1\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                _ => panic!("unexpected command: {args:?}"),
            }
        }
    }

    #[tokio::test]
    async fn ensure_session_returns_existing() {
        let name = SessionName::try_from("existing").unwrap();
        let session = ensure_session(&MockAlreadyExists, &name).await.unwrap();
        assert_eq!(session.name, "existing");
        assert_eq!(session.windows, 3);
    }

    #[tokio::test]
    async fn ensure_session_validates_name() {
        // With newtypes, validation happens at construction time
        assert!(SessionName::try_from("").is_err());
    }
}
