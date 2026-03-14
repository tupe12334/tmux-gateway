use std::fmt;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
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

    #[error("failed to parse tmux output for {command}: {details}")]
    ParseError { command: String, details: String },
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
            Self::InvalidTarget(_) | Self::ParseError { .. } => 400,
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
            Self::InvalidTarget(_) | Self::ParseError { .. } => GrpcCode::InvalidArgument,
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
        assert!(
            matches!(err, TmuxError::CommandFailed { ref command, ref stderr } if command == "new-session" && stderr == "some unknown error")
        );
    }

    // --- TmuxNotRunning ---

    #[test]
    fn from_stderr_no_server_running() {
        let err = TmuxError::from_stderr(
            "list-sessions",
            "no server running on /tmp/tmux-1000/default",
            "",
        );
        assert!(matches!(err, TmuxError::TmuxNotRunning));
    }

    #[test]
    fn from_stderr_no_server_running_case_insensitive() {
        let err = TmuxError::from_stderr("list-sessions", "No Server Running", "");
        assert!(matches!(err, TmuxError::TmuxNotRunning));
    }

    // --- SessionNotFound ---

    #[test]
    fn from_stderr_session_not_found() {
        let err = TmuxError::from_stderr("attach", "session not found: mysess", "mysess");
        assert!(matches!(err, TmuxError::SessionNotFound(ref t) if t == "mysess"));
    }

    #[test]
    fn from_stderr_cant_find_session() {
        let err = TmuxError::from_stderr("attach", "can't find session: mysess", "mysess");
        assert!(matches!(err, TmuxError::SessionNotFound(ref t) if t == "mysess"));
    }

    #[test]
    fn from_stderr_session_not_found_mixed_case() {
        let err = TmuxError::from_stderr("attach", "Session Not Found: foo", "foo");
        assert!(matches!(err, TmuxError::SessionNotFound(ref t) if t == "foo"));
    }

    #[test]
    fn from_stderr_cant_find_session_upper_case() {
        let err = TmuxError::from_stderr("attach", "CAN'T FIND SESSION: bar", "bar");
        assert!(matches!(err, TmuxError::SessionNotFound(ref t) if t == "bar"));
    }

    // --- WindowNotFound ---

    #[test]
    fn from_stderr_window_not_found() {
        let err = TmuxError::from_stderr("select-window", "window not found: mywin", "mywin");
        assert!(matches!(err, TmuxError::WindowNotFound(ref t) if t == "mywin"));
    }

    #[test]
    fn from_stderr_cant_find_window() {
        let err = TmuxError::from_stderr("select-window", "can't find window: mywin", "mywin");
        assert!(matches!(err, TmuxError::WindowNotFound(ref t) if t == "mywin"));
    }

    #[test]
    fn from_stderr_window_not_found_mixed_case() {
        let err = TmuxError::from_stderr("select-window", "Window Not Found: w1", "w1");
        assert!(matches!(err, TmuxError::WindowNotFound(ref t) if t == "w1"));
    }

    // --- PaneNotFound ---

    #[test]
    fn from_stderr_pane_not_found() {
        let err = TmuxError::from_stderr("select-pane", "pane not found: %1", "%1");
        assert!(matches!(err, TmuxError::PaneNotFound(ref t) if t == "%1"));
    }

    #[test]
    fn from_stderr_cant_find_pane() {
        let err = TmuxError::from_stderr("select-pane", "can't find pane: %1", "%1");
        assert!(matches!(err, TmuxError::PaneNotFound(ref t) if t == "%1"));
    }

    #[test]
    fn from_stderr_pane_not_found_mixed_case() {
        let err = TmuxError::from_stderr("select-pane", "PANE NOT FOUND: %2", "%2");
        assert!(matches!(err, TmuxError::PaneNotFound(ref t) if t == "%2"));
    }

    // --- Target preservation ---

    #[test]
    fn from_stderr_preserves_target_in_session_not_found() {
        let err = TmuxError::from_stderr("cmd", "session not found", "my-target:0.1");
        assert!(matches!(err, TmuxError::SessionNotFound(ref t) if t == "my-target:0.1"));
    }

    #[test]
    fn from_stderr_preserves_target_in_window_not_found() {
        let err = TmuxError::from_stderr("cmd", "window not found", "sess:42");
        assert!(matches!(err, TmuxError::WindowNotFound(ref t) if t == "sess:42"));
    }

    #[test]
    fn from_stderr_preserves_target_in_pane_not_found() {
        let err = TmuxError::from_stderr("cmd", "pane not found", "sess:0.%3");
        assert!(matches!(err, TmuxError::PaneNotFound(ref t) if t == "sess:0.%3"));
    }

    // --- Edge cases ---

    #[test]
    fn from_stderr_empty_stderr_falls_through() {
        let err = TmuxError::from_stderr("cmd", "", "target");
        assert!(
            matches!(err, TmuxError::CommandFailed { ref command, ref stderr } if command == "cmd" && stderr.is_empty())
        );
    }

    #[test]
    fn from_stderr_whitespace_only_stderr_falls_through() {
        let err = TmuxError::from_stderr("cmd", "   \n  ", "target");
        assert!(matches!(err, TmuxError::CommandFailed { ref command, .. } if command == "cmd"));
    }

    #[test]
    fn from_stderr_trims_stderr_in_command_failed() {
        let err = TmuxError::from_stderr("cmd", "  some error  \n", "target");
        assert!(
            matches!(err, TmuxError::CommandFailed { ref stderr, .. } if stderr == "some error")
        );
    }

    #[test]
    fn from_stderr_command_preserved_in_command_failed() {
        let err = TmuxError::from_stderr("kill-session", "unexpected error", "");
        assert!(
            matches!(err, TmuxError::CommandFailed { ref command, .. } if command == "kill-session")
        );
    }

    // --- Priority: no server running takes precedence ---

    #[test]
    fn from_stderr_no_server_running_takes_precedence() {
        // If stderr contains both "no server running" and "session not found",
        // TmuxNotRunning should win because it's checked first.
        let err = TmuxError::from_stderr("cmd", "no server running on... session not found", "s");
        assert!(matches!(err, TmuxError::TmuxNotRunning));
    }
}
