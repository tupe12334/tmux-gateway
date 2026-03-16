use std::time::Duration;

use crate::TmuxError;

/// Domain concept: deadline for a tmux operation.
///
/// Different operations may warrant different timeouts — a health check should
/// complete quickly while a large `capture-pane` can take longer. Callers pass
/// an `OperationTimeout` to [`execute_with_timeout`] to express this intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationTimeout(pub Duration);

impl OperationTimeout {
    /// Create a timeout with the given duration.
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Create a timeout from seconds.
    pub fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// Return the inner duration.
    pub fn duration(&self) -> Duration {
        self.0
    }
}

impl Default for OperationTimeout {
    fn default() -> Self {
        Self(Duration::from_secs(10))
    }
}

impl From<Duration> for OperationTimeout {
    fn from(d: Duration) -> Self {
        Self(d)
    }
}

/// Execute a blocking task with a domain-level timeout.
///
/// Wraps `tokio::task::spawn_blocking` with `tokio::time::timeout` so that
/// a hung tmux process cannot block the thread pool indefinitely. Returns
/// [`TmuxError::Timeout`] when the deadline is exceeded.
pub async fn execute_with_timeout<T>(
    timeout: OperationTimeout,
    task: impl FnOnce() -> Result<T, TmuxError> + Send + 'static,
) -> Result<T, TmuxError>
where
    T: Send + 'static,
{
    tokio::time::timeout(timeout.0, tokio::task::spawn_blocking(task))
        .await
        .map_err(|_| TmuxError::Timeout {
            command: String::new(),
            timeout: timeout.0,
        })?
        .map_err(|e| TmuxError::CommandFailed {
            command: "spawn_blocking".to_string(),
            stderr: format!("task join error: {e}"),
        })?
}

/// Output from a tmux command execution.
#[derive(Debug, Clone)]
pub struct TmuxOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Port trait for executing tmux commands.
///
/// Domain operations depend on this trait rather than any concrete tmux
/// implementation, enabling mock executors for testing and alternative
/// backends (remote tmux over SSH, recording/replay, etc.).
pub trait TmuxExecutor: Send + Sync {
    fn execute(
        &self,
        args: &[&str],
    ) -> impl std::future::Future<Output = Result<TmuxOutput, TmuxError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_timeout_default_is_10_seconds() {
        let t = OperationTimeout::default();
        assert_eq!(t.duration(), Duration::from_secs(10));
    }

    #[test]
    fn operation_timeout_from_secs() {
        let t = OperationTimeout::from_secs(5);
        assert_eq!(t.duration(), Duration::from_secs(5));
    }

    #[test]
    fn operation_timeout_new() {
        let t = OperationTimeout::new(Duration::from_millis(500));
        assert_eq!(t.duration(), Duration::from_millis(500));
    }

    #[test]
    fn operation_timeout_from_duration() {
        let t: OperationTimeout = Duration::from_secs(3).into();
        assert_eq!(t.duration(), Duration::from_secs(3));
    }

    #[tokio::test]
    async fn execute_with_timeout_succeeds_within_deadline() {
        let result = execute_with_timeout(OperationTimeout::from_secs(5), || Ok(42)).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn execute_with_timeout_propagates_error() {
        let result: Result<(), _> = execute_with_timeout(OperationTimeout::from_secs(5), || {
            Err(TmuxError::TmuxNotRunning)
        })
        .await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    #[tokio::test]
    async fn execute_with_timeout_returns_timeout_on_exceeded_deadline() {
        let result = execute_with_timeout(OperationTimeout::new(Duration::from_millis(1)), || {
            std::thread::sleep(Duration::from_secs(5));
            Ok(())
        })
        .await;
        assert!(matches!(result, Err(TmuxError::Timeout { .. })));
    }
}
