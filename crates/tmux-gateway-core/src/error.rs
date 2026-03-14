use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum TmuxError {
    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("window not found: {0}")]
    WindowNotFound(String),

    #[error("pane not found: {0}")]
    PaneNotFound(String),

    #[error("tmux server is not running")]
    TmuxNotRunning,

    #[error("session already exists: {0}")]
    SessionAlreadyExists(String),

    #[error("tmux command failed: {command}: {stderr}")]
    CommandFailed { command: String, stderr: String },

    #[error("invalid target: {0}")]
    InvalidTarget(String),
}

impl TmuxError {
    /// Classify a tmux stderr output into the appropriate error variant.
    pub(crate) fn from_stderr(command: &str, stderr: &str, target: &str) -> Self {
        let stderr_lower = stderr.to_lowercase();

        if stderr_lower.contains("no server running") {
            return Self::TmuxNotRunning;
        }

        if stderr_lower.contains("session not found") || stderr_lower.contains("can't find session")
        {
            return Self::SessionNotFound(target.to_string());
        }

        if stderr_lower.contains("window not found") || stderr_lower.contains("can't find window") {
            return Self::WindowNotFound(target.to_string());
        }

        if stderr_lower.contains("pane not found") || stderr_lower.contains("can't find pane") {
            return Self::PaneNotFound(target.to_string());
        }

        if stderr_lower.contains("duplicate session") {
            return Self::SessionAlreadyExists(target.to_string());
        }

        Self::CommandFailed {
            command: command.to_string(),
            stderr: stderr.trim().to_string(),
        }
    }

    /// Returns the appropriate HTTP status code for this error.
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::SessionNotFound(_) | Self::WindowNotFound(_) | Self::PaneNotFound(_) => 404,
            Self::SessionAlreadyExists(_) => 409,
            Self::InvalidTarget(_) => 400,
            Self::TmuxNotRunning | Self::CommandFailed { .. } => 500,
        }
    }

    /// Returns a string suitable for use as a gRPC status code.
    pub fn grpc_code(&self) -> GrpcCode {
        match self {
            Self::SessionNotFound(_) | Self::WindowNotFound(_) | Self::PaneNotFound(_) => {
                GrpcCode::NotFound
            }
            Self::SessionAlreadyExists(_) => GrpcCode::AlreadyExists,
            Self::InvalidTarget(_) => GrpcCode::InvalidArgument,
            Self::TmuxNotRunning | Self::CommandFailed { .. } => GrpcCode::Internal,
        }
    }
}

impl From<crate::validation::ValidationError> for TmuxError {
    fn from(e: crate::validation::ValidationError) -> Self {
        Self::InvalidTarget(e.to_string())
    }
}

/// Subset of gRPC status codes relevant to tmux errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcCode {
    NotFound,
    AlreadyExists,
    InvalidArgument,
    Internal,
}

impl fmt::Display for GrpcCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "NOT_FOUND"),
            Self::AlreadyExists => write!(f, "ALREADY_EXISTS"),
            Self::InvalidArgument => write!(f, "INVALID_ARGUMENT"),
            Self::Internal => write!(f, "INTERNAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_stderr_duplicate_session() {
        let err = TmuxError::from_stderr("new-session", "duplicate session: foo", "foo");
        assert!(matches!(err, TmuxError::SessionAlreadyExists(ref name) if name == "foo"));
        assert_eq!(err.to_string(), "session already exists: foo");
    }

    #[test]
    fn from_stderr_duplicate_session_case_insensitive() {
        let err = TmuxError::from_stderr("new-session", "Duplicate Session: bar", "bar");
        assert!(matches!(err, TmuxError::SessionAlreadyExists(ref name) if name == "bar"));
    }

    #[test]
    fn session_already_exists_http_status() {
        let err = TmuxError::SessionAlreadyExists("test".to_string());
        assert_eq!(err.http_status_code(), 409);
    }

    #[test]
    fn session_already_exists_grpc_code() {
        let err = TmuxError::SessionAlreadyExists("test".to_string());
        assert_eq!(err.grpc_code(), GrpcCode::AlreadyExists);
        assert_eq!(GrpcCode::AlreadyExists.to_string(), "ALREADY_EXISTS");
    }

    #[test]
    fn from_stderr_unknown_falls_through_to_command_failed() {
        let err = TmuxError::from_stderr("new-session", "some unknown error", "test");
        assert!(matches!(err, TmuxError::CommandFailed { .. }));
    }
}
