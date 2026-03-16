use std::fmt;

use crate::options::OptionScope;

const MAX_SESSION_NAME_LEN: usize = 128;
const MAX_TARGET_LEN: usize = 256;
const MAX_OPTION_NAME_LEN: usize = 128;
const MAX_COMMAND_LEN: usize = 1024;
const MAX_ENV_VAR_NAME_LEN: usize = 256;
const MAX_ENV_VAR_VALUE_LEN: usize = 4096;
const MAX_BUFFER_NAME_LEN: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyInput { field: &'static str },
    InvalidSessionName { reason: String },
    InvalidWindowName { reason: String },
    InvalidTarget { reason: String },
    InvalidOptionName { reason: String },
    InvalidCommand { reason: String },
    InvalidEnvVarName { reason: String },
    InvalidEnvVarValue { reason: String },
    InvalidBufferName { reason: String },
}

impl fmt::Display for ValidationError {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput { field } => write!(f, "{field} must not be empty"),
            Self::InvalidSessionName { reason } => {
                write!(f, "invalid session name: {reason}")
            }
            Self::InvalidWindowName { reason } => {
                write!(f, "invalid window name: {reason}")
            }
            Self::InvalidTarget { reason } => write!(f, "invalid target: {reason}"),
            Self::InvalidOptionName { reason } => {
                write!(f, "invalid option name: {reason}")
            }
            Self::InvalidCommand { reason } => {
                write!(f, "invalid command: {reason}")
            }
            Self::InvalidEnvVarName { reason } => {
                write!(f, "invalid environment variable name: {reason}")
            }
            Self::InvalidEnvVarValue { reason } => {
                write!(f, "invalid environment variable value: {reason}")
            }
            Self::InvalidBufferName { reason } => {
                write!(f, "invalid buffer name: {reason}")
            }
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

/// Validate a window name for creation or renaming.
/// Allowed: alphanumeric, hyphens, underscores, dots. 1-128 chars.
pub fn validate_window_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyInput { field: "name" });
    }
    if name.len() > MAX_SESSION_NAME_LEN {
        return Err(ValidationError::InvalidWindowName {
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
        return Err(ValidationError::InvalidWindowName {
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
    if target.len() > MAX_TARGET_LEN {
        return Err(ValidationError::InvalidTarget {
            reason: format!(
                "must be at most {MAX_TARGET_LEN} characters, got {}",
                target.len()
            ),
        });
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
    if target.len() > MAX_TARGET_LEN {
        return Err(ValidationError::InvalidTarget {
            reason: format!(
                "must be at most {MAX_TARGET_LEN} characters, got {}",
                target.len()
            ),
        });
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
    if target.len() > MAX_TARGET_LEN {
        return Err(ValidationError::InvalidTarget {
            reason: format!(
                "must be at most {MAX_TARGET_LEN} characters, got {}",
                target.len()
            ),
        });
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

/// Validate a tmux option name.
/// Allowed: alphanumeric, hyphens, underscores. 1-128 chars.
pub fn validate_option_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyInput { field: "name" });
    }
    if name.len() > MAX_OPTION_NAME_LEN {
        return Err(ValidationError::InvalidOptionName {
            reason: format!(
                "must be at most {MAX_OPTION_NAME_LEN} characters, got {}",
                name.len()
            ),
        });
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ValidationError::InvalidOptionName {
            reason: "must contain only alphanumeric characters, hyphens, or underscores"
                .to_string(),
        });
    }
    Ok(())
}

/// Validate a target for an option scope.
/// Global scope requires no target, session scope requires a session name,
/// window scope requires a window target (session:window).
pub fn validate_option_scope_target(
    scope: OptionScope,
    target: &str,
) -> Result<(), ValidationError> {
    match scope {
        OptionScope::Global => Ok(()),
        OptionScope::Session => validate_session_target(target),
        OptionScope::Window => validate_window_target(target),
    }
}

/// Validate a command string for safety.
/// Allowed: alphanumeric, spaces, hyphens, underscores, dots, forward slashes,
/// tildes, equals, colons, commas, plus, at signs. Rejects shell metacharacters.
pub fn validate_command(command: &str) -> Result<(), ValidationError> {
    if command.is_empty() {
        return Err(ValidationError::EmptyInput { field: "command" });
    }
    if command.len() > MAX_COMMAND_LEN {
        return Err(ValidationError::InvalidCommand {
            reason: format!(
                "must be at most {MAX_COMMAND_LEN} characters, got {}",
                command.len()
            ),
        });
    }
    if !command.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(
                c,
                ' ' | '-' | '_' | '.' | '/' | '~' | '=' | ':' | ',' | '+' | '@'
            )
    }) {
        return Err(ValidationError::InvalidCommand {
            reason: "must not contain shell metacharacters (;|&`$(){}\\<>!\"'#*?\n etc.)"
                .to_string(),
        });
    }
    Ok(())
}

/// Validate an environment variable name.
/// Must start with a letter or underscore, then alphanumeric or underscores. 1-256 chars.
pub fn validate_env_var_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyInput { field: "name" });
    }
    if name.len() > MAX_ENV_VAR_NAME_LEN {
        return Err(ValidationError::InvalidEnvVarName {
            reason: format!(
                "must be at most {MAX_ENV_VAR_NAME_LEN} characters, got {}",
                name.len()
            ),
        });
    }
    let Some(first) = name.chars().next() else {
        return Err(ValidationError::EmptyInput { field: "name" });
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(ValidationError::InvalidEnvVarName {
            reason: "must start with a letter or underscore".to_string(),
        });
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(ValidationError::InvalidEnvVarName {
            reason: "must contain only alphanumeric characters or underscores".to_string(),
        });
    }
    Ok(())
}

/// Validate an environment variable value.
/// Must not contain null bytes. Max 4096 chars.
pub fn validate_env_var_value(value: &str) -> Result<(), ValidationError> {
    if value.len() > MAX_ENV_VAR_VALUE_LEN {
        return Err(ValidationError::InvalidEnvVarValue {
            reason: format!(
                "must be at most {MAX_ENV_VAR_VALUE_LEN} characters, got {}",
                value.len()
            ),
        });
    }
    if value.contains('\0') {
        return Err(ValidationError::InvalidEnvVarValue {
            reason: "must not contain null bytes".to_string(),
        });
    }
    Ok(())
}

/// Validate a paste buffer name.
/// Allowed: alphanumeric, underscores. 1-128 chars.
pub fn validate_buffer_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::EmptyInput { field: "name" });
    }
    if name.len() > MAX_BUFFER_NAME_LEN {
        return Err(ValidationError::InvalidBufferName {
            reason: format!(
                "must be at most {MAX_BUFFER_NAME_LEN} characters, got {}",
                name.len()
            ),
        });
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(ValidationError::InvalidBufferName {
            reason: "must contain only alphanumeric characters or underscores".to_string(),
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
            reason:
                "must contain only alphanumeric characters, hyphens, underscores, dots, or colons"
                    .to_string(),
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

    // ── Window name validation ──

    #[test]
    fn valid_window_names() {
        assert!(validate_window_name("my-window").is_ok());
        assert!(validate_window_name("test_123").is_ok());
        assert!(validate_window_name("a").is_ok());
        assert!(validate_window_name("My.Window").is_ok());
    }

    #[test]
    fn empty_window_name() {
        assert_eq!(
            validate_window_name(""),
            Err(ValidationError::EmptyInput { field: "name" })
        );
    }

    #[test]
    fn window_name_too_long() {
        let long = "a".repeat(129);
        assert!(matches!(
            validate_window_name(&long),
            Err(ValidationError::InvalidWindowName { .. })
        ));
    }

    #[test]
    fn window_name_with_special_chars() {
        assert!(validate_window_name("foo;bar").is_err());
        assert!(validate_window_name("foo&bar").is_err());
        assert!(validate_window_name("$(cmd)").is_err());
    }

    // ── Target max-length validation ──

    #[test]
    fn session_target_at_max_length() {
        let target = "a".repeat(MAX_TARGET_LEN);
        assert!(validate_session_target(&target).is_ok());
    }

    #[test]
    fn session_target_over_max_length() {
        let target = "a".repeat(MAX_TARGET_LEN + 1);
        assert!(matches!(
            validate_session_target(&target),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    #[test]
    fn window_target_at_max_length() {
        let session = "a".repeat(MAX_TARGET_LEN - 2);
        let target = format!("{session}:w");
        assert_eq!(target.len(), MAX_TARGET_LEN);
        assert!(validate_window_target(&target).is_ok());
    }

    #[test]
    fn window_target_over_max_length() {
        let session = "a".repeat(MAX_TARGET_LEN);
        let target = format!("{session}:w");
        assert!(matches!(
            validate_window_target(&target),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    #[test]
    fn pane_target_at_max_length() {
        let session = "a".repeat(MAX_TARGET_LEN - 4);
        let target = format!("{session}:w.0");
        assert_eq!(target.len(), MAX_TARGET_LEN);
        assert!(validate_pane_target(&target).is_ok());
    }

    #[test]
    fn pane_target_over_max_length() {
        let session = "a".repeat(MAX_TARGET_LEN);
        let target = format!("{session}:w.0");
        assert!(matches!(
            validate_pane_target(&target),
            Err(ValidationError::InvalidTarget { .. })
        ));
    }

    // ── Command injection prevention ──

    #[test]
    fn rejects_shell_metacharacters() {
        assert!(validate_session_target("$(whoami)").is_err());
        assert!(validate_window_target("sess;rm -rf:0").is_err());
        assert!(validate_pane_target("s:w.0 && echo").is_err());
    }

    // ── Option name validation ──

    #[test]
    fn valid_option_names() {
        assert!(validate_option_name("status").is_ok());
        assert!(validate_option_name("base-index").is_ok());
        assert!(validate_option_name("default_terminal").is_ok());
        assert!(validate_option_name("a").is_ok());
    }

    #[test]
    fn empty_option_name() {
        assert_eq!(
            validate_option_name(""),
            Err(ValidationError::EmptyInput { field: "name" })
        );
    }

    #[test]
    fn option_name_too_long() {
        let long = "a".repeat(129);
        assert!(matches!(
            validate_option_name(&long),
            Err(ValidationError::InvalidOptionName { .. })
        ));
    }

    #[test]
    fn option_name_with_special_chars() {
        assert!(validate_option_name("foo;bar").is_err());
        assert!(validate_option_name("foo bar").is_err());
        assert!(validate_option_name("$(cmd)").is_err());
        assert!(validate_option_name("foo.bar").is_err());
    }

    // ── Option scope target validation ──

    #[test]
    fn option_scope_target_global_accepts_any() {
        assert!(validate_option_scope_target(OptionScope::Global, "anything").is_ok());
    }

    #[test]
    fn option_scope_target_session_valid() {
        assert!(validate_option_scope_target(OptionScope::Session, "my-session").is_ok());
    }

    #[test]
    fn option_scope_target_session_invalid() {
        assert!(validate_option_scope_target(OptionScope::Session, "").is_err());
    }

    #[test]
    fn option_scope_target_window_valid() {
        assert!(validate_option_scope_target(OptionScope::Window, "sess:0").is_ok());
    }

    #[test]
    fn option_scope_target_window_invalid() {
        assert!(validate_option_scope_target(OptionScope::Window, "sess").is_err());
    }

    // ── Command validation ──

    #[test]
    fn valid_commands() {
        assert!(validate_command("vim").is_ok());
        assert!(validate_command("htop").is_ok());
        assert!(validate_command("tail -f /var/log/syslog").is_ok());
        assert!(validate_command("python3 script.py --arg=value").is_ok());
        assert!(validate_command("/usr/bin/top").is_ok());
    }

    #[test]
    fn empty_command() {
        assert_eq!(
            validate_command(""),
            Err(ValidationError::EmptyInput { field: "command" })
        );
    }

    #[test]
    fn command_too_long() {
        let long = "a".repeat(MAX_COMMAND_LEN + 1);
        assert!(matches!(
            validate_command(&long),
            Err(ValidationError::InvalidCommand { .. })
        ));
    }

    #[test]
    fn command_rejects_shell_metacharacters() {
        assert!(validate_command("echo; rm -rf /").is_err());
        assert!(validate_command("cat | grep foo").is_err());
        assert!(validate_command("cmd && other").is_err());
        assert!(validate_command("$(whoami)").is_err());
        assert!(validate_command("`whoami`").is_err());
        assert!(validate_command("cmd > file").is_err());
        assert!(validate_command("cmd < file").is_err());
    }

    // ── Environment variable name validation ──

    #[test]
    fn valid_env_var_names() {
        assert!(validate_env_var_name("PATH").is_ok());
        assert!(validate_env_var_name("HOME").is_ok());
        assert!(validate_env_var_name("_PRIVATE").is_ok());
        assert!(validate_env_var_name("MY_VAR_123").is_ok());
        assert!(validate_env_var_name("a").is_ok());
    }

    #[test]
    fn empty_env_var_name() {
        assert_eq!(
            validate_env_var_name(""),
            Err(ValidationError::EmptyInput { field: "name" })
        );
    }

    #[test]
    fn env_var_name_too_long() {
        let long = "A".repeat(MAX_ENV_VAR_NAME_LEN + 1);
        assert!(matches!(
            validate_env_var_name(&long),
            Err(ValidationError::InvalidEnvVarName { .. })
        ));
    }

    #[test]
    fn env_var_name_starts_with_digit() {
        assert!(matches!(
            validate_env_var_name("1VAR"),
            Err(ValidationError::InvalidEnvVarName { .. })
        ));
    }

    #[test]
    fn env_var_name_with_special_chars() {
        assert!(validate_env_var_name("MY-VAR").is_err());
        assert!(validate_env_var_name("MY.VAR").is_err());
        assert!(validate_env_var_name("MY VAR").is_err());
        assert!(validate_env_var_name("$(cmd)").is_err());
        assert!(validate_env_var_name("foo;bar").is_err());
    }

    // ── Environment variable value validation ──

    #[test]
    fn valid_env_var_values() {
        assert!(validate_env_var_value("").is_ok());
        assert!(validate_env_var_value("/usr/bin:/usr/local/bin").is_ok());
        assert!(validate_env_var_value("hello world").is_ok());
        assert!(validate_env_var_value("value with spaces and symbols!@#$%").is_ok());
    }

    #[test]
    fn env_var_value_too_long() {
        let long = "a".repeat(MAX_ENV_VAR_VALUE_LEN + 1);
        assert!(matches!(
            validate_env_var_value(&long),
            Err(ValidationError::InvalidEnvVarValue { .. })
        ));
    }

    #[test]
    fn env_var_value_with_null_byte() {
        assert!(matches!(
            validate_env_var_value("val\0ue"),
            Err(ValidationError::InvalidEnvVarValue { .. })
        ));
    }

    // ── Buffer name validation ──

    #[test]
    fn valid_buffer_names() {
        assert!(validate_buffer_name("buffer0").is_ok());
        assert!(validate_buffer_name("my_buffer").is_ok());
        assert!(validate_buffer_name("a").is_ok());
        assert!(validate_buffer_name("Buffer123").is_ok());
    }

    #[test]
    fn empty_buffer_name() {
        assert_eq!(
            validate_buffer_name(""),
            Err(ValidationError::EmptyInput { field: "name" })
        );
    }

    #[test]
    fn buffer_name_too_long() {
        let long = "a".repeat(129);
        assert!(matches!(
            validate_buffer_name(&long),
            Err(ValidationError::InvalidBufferName { .. })
        ));
    }

    #[test]
    fn buffer_name_with_special_chars() {
        assert!(validate_buffer_name("buf;evil").is_err());
        assert!(validate_buffer_name("buf&evil").is_err());
        assert!(validate_buffer_name("$(cmd)").is_err());
        assert!(validate_buffer_name("buf-name").is_err());
        assert!(validate_buffer_name("buf.name").is_err());
        assert!(validate_buffer_name("buf name").is_err());
    }
}
