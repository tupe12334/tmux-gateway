use std::fmt;

const MAX_SESSION_NAME_LEN: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyInput { field: &'static str },
    InvalidSessionName { reason: String },
    InvalidTarget { reason: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput { field } => write!(f, "{field} must not be empty"),
            Self::InvalidSessionName { reason } => {
                write!(f, "invalid session name: {reason}")
            }
            Self::InvalidTarget { reason } => write!(f, "invalid target: {reason}"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a session name for creation.
/// Allowed: alphanumeric, hyphens, underscores, dots. 1-128 chars.
pub fn validate_session_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyInput { field: "name" });
    }
    if name.len() > MAX_SESSION_NAME_LEN {
        return Err(ValidationError::InvalidSessionName {
            reason: format!(
                "must be at most {MAX_SESSION_NAME_LEN} characters, got {}",
                name.len()
            ),
        });
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(ValidationError::InvalidSessionName {
            reason: "must contain only alphanumeric characters, hyphens, underscores, or dots"
                .to_string(),
        });
    }
    Ok(())
}

/// Validate a target identifier used in kill-session.
/// Format: a valid session name.
pub fn validate_session_target(target: &str) -> Result<(), ValidationError> {
    if target.is_empty() {
        return Err(ValidationError::EmptyInput { field: "target" });
    }
    validate_target_chars(target)?;
    // Session target is just a session name — no colons or dots required
    if target.contains(':') || target.contains('.') {
        return Err(ValidationError::InvalidTarget {
            reason: "session target must not contain ':' or '.' — use kill-window or kill-pane for sub-session targets".to_string(),
        });
    }
    Ok(())
}

/// Validate a target identifier used in kill-window.
/// Format: `session:window` where window is a name or index.
pub fn validate_window_target(target: &str) -> Result<(), ValidationError> {
    if target.is_empty() {
        return Err(ValidationError::EmptyInput { field: "target" });
    }
    validate_target_chars(target)?;
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        return Err(ValidationError::InvalidTarget {
            reason: "window target must be in format 'session:window'".to_string(),
        });
    }
    if parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError::InvalidTarget {
            reason: "session and window parts must not be empty".to_string(),
        });
    }
    Ok(())
}

/// Validate a target identifier used in kill-pane.
/// Format: `session:window.pane` where pane is an index.
pub fn validate_pane_target(target: &str) -> Result<(), ValidationError> {
    if target.is_empty() {
        return Err(ValidationError::EmptyInput { field: "target" });
    }
    validate_target_chars(target)?;
    // Must contain both : and .
    let Some(colon_pos) = target.find(':') else {
        return Err(ValidationError::InvalidTarget {
            reason: "pane target must be in format 'session:window.pane'".to_string(),
        });
    };
    let after_colon = &target[colon_pos + 1..];
    let Some(dot_pos) = after_colon.find('.') else {
        return Err(ValidationError::InvalidTarget {
            reason: "pane target must be in format 'session:window.pane'".to_string(),
        });
    };
    let session = &target[..colon_pos];
    let window = &after_colon[..dot_pos];
    let pane = &after_colon[dot_pos + 1..];
    if session.is_empty() || window.is_empty() || pane.is_empty() {
        return Err(ValidationError::InvalidTarget {
            reason: "session, window, and pane parts must not be empty".to_string(),
        });
    }
    Ok(())
}

/// Ensure target contains only safe characters (prevent command injection).
fn validate_target_chars(target: &str) -> Result<(), ValidationError> {
    if !target
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err(ValidationError::InvalidTarget {
            reason: "must contain only alphanumeric characters, hyphens, underscores, dots, or colons".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Session name validation ──

    #[test]
    fn valid_session_names() {
        assert!(validate_session_name("my-session").is_ok());
        assert!(validate_session_name("test_123").is_ok());
        assert!(validate_session_name("a").is_ok());
        assert!(validate_session_name("My.Session").is_ok());
    }

    #[test]
    fn empty_session_name() {
        assert_eq!(
            validate_session_name(""),
            Err(ValidationError::EmptyInput { field: "name" })
        );
    }

    #[test]
    fn session_name_too_long() {
        let long = "a".repeat(129);
        assert!(matches!(
            validate_session_name(&long),
            Err(ValidationError::InvalidSessionName { .. })
        ));
    }

    #[test]
    fn session_name_with_spaces() {
        assert!(matches!(
            validate_session_name("my session"),
            Err(ValidationError::InvalidSessionName { .. })
        ));
    }

    #[test]
    fn session_name_with_special_chars() {
        assert!(validate_session_name("foo;bar").is_err());
        assert!(validate_session_name("foo&bar").is_err());
        assert!(validate_session_name("$(cmd)").is_err());
        assert!(validate_session_name("foo\nbar").is_err());
    }

    // ── Session target validation ──

    #[test]
    fn valid_session_targets() {
        assert!(validate_session_target("my-session").is_ok());
        assert!(validate_session_target("test_123").is_ok());
    }

    #[test]
    fn empty_session_target() {
        assert_eq!(
            validate_session_target(""),
            Err(ValidationError::EmptyInput { field: "target" })
        );
    }

    #[test]
    fn session_target_with_colon() {
        assert!(matches!(
            validate_session_target("sess:win"),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    // ── Window target validation ──

    #[test]
    fn valid_window_targets() {
        assert!(validate_window_target("sess:0").is_ok());
        assert!(validate_window_target("my-session:my-window").is_ok());
        assert!(validate_window_target("s:1").is_ok());
    }

    #[test]
    fn empty_window_target() {
        assert_eq!(
            validate_window_target(""),
            Err(ValidationError::EmptyInput { field: "target" })
        );
    }

    #[test]
    fn window_target_missing_colon() {
        assert!(matches!(
            validate_window_target("session"),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    #[test]
    fn window_target_empty_parts() {
        assert!(validate_window_target(":window").is_err());
        assert!(validate_window_target("session:").is_err());
    }

    // ── Pane target validation ──

    #[test]
    fn valid_pane_targets() {
        assert!(validate_pane_target("sess:0.1").is_ok());
        assert!(validate_pane_target("my-session:my-window.0").is_ok());
    }

    #[test]
    fn empty_pane_target() {
        assert_eq!(
            validate_pane_target(""),
            Err(ValidationError::EmptyInput { field: "target" })
        );
    }

    #[test]
    fn pane_target_missing_dot() {
        assert!(matches!(
            validate_pane_target("sess:0"),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    #[test]
    fn pane_target_missing_colon() {
        assert!(matches!(
            validate_pane_target("sess.0"),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    #[test]
    fn pane_target_empty_parts() {
        assert!(validate_pane_target(":win.0").is_err());
        assert!(validate_pane_target("sess:.0").is_err());
        assert!(validate_pane_target("sess:win.").is_err());
    }

    // ── Command injection prevention ──

    #[test]
    fn rejects_shell_metacharacters() {
        assert!(validate_session_target("$(whoami)").is_err());
        assert!(validate_window_target("sess;rm -rf:0").is_err());
        assert!(validate_pane_target("s:w.0 && echo").is_err());
    }
}
