use crate::executor::TmuxExecutor;

use super::TmuxError;

/// Options for controlling what content is captured from a pane.
#[derive(Debug, Clone, Default)]
pub struct CaptureOptions {
    /// Starting line number (-S flag). Negative values reach into scroll history.
    pub start_line: Option<i32>,
    /// Ending line number (-E flag).
    pub end_line: Option<i32>,
    /// Include escape sequences in output (-e flag).
    pub escape_sequences: bool,
}

/// A domain type representing normalized pane content.
///
/// Normalization contract:
/// - Trailing whitespace on each line is trimmed
/// - Trailing blank lines (from tmux pane height padding) are removed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedContent(String);

impl CapturedContent {
    pub fn new(raw: &str) -> Self {
        Self(normalize_pane_content(raw))
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CapturedContent {
    #[allow(no_wrapper_functions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<CapturedContent> for String {
    fn from(c: CapturedContent) -> Self {
        c.0
    }
}

impl AsRef<str> for CapturedContent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Normalize raw tmux capture-pane output.
///
/// This is a pure function that:
/// - Trims trailing whitespace from each line
/// - Removes trailing blank lines (tmux pads output to pane height)
pub fn normalize_pane_content(raw: &str) -> String {
    let trimmed_lines: Vec<&str> = raw.lines().map(|line| line.trim_end()).collect();
    let last_non_empty = trimmed_lines.iter().rposition(|line| !line.is_empty());
    match last_non_empty {
        Some(idx) => trimmed_lines[..=idx].join("\n"),
        None => String::new(),
    }
}

#[tracing::instrument(skip(executor))]
pub async fn capture_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<String, TmuxError> {
    capture_pane_with_options(executor, target, &CaptureOptions::default()).await
}

#[tracing::instrument(skip(executor))]
pub async fn capture_pane_with_options(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    opts: &CaptureOptions,
) -> Result<String, TmuxError> {
    let mut args: Vec<&str> = vec!["capture-pane", "-p", "-t", target];

    let start_str;
    if let Some(start) = opts.start_line {
        start_str = start.to_string();
        args.push("-S");
        args.push(&start_str);
    }

    let end_str;
    if let Some(end) = opts.end_line {
        end_str = end.to_string();
        args.push("-E");
        args.push(&end_str);
    }

    if opts.escape_sequences {
        args.push("-e");
    }

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "capture-pane",
            &output.stderr,
            target,
        ));
    }
    Ok(normalize_pane_content(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_trailing_blank_lines() {
        let raw = "hello\nworld\n\n\n\n";
        assert_eq!(normalize_pane_content(raw), "hello\nworld");
    }

    #[test]
    fn normalize_strips_trailing_whitespace_per_line() {
        let raw = "hello   \nworld  \n";
        assert_eq!(normalize_pane_content(raw), "hello\nworld");
    }

    #[test]
    fn normalize_empty_pane() {
        assert_eq!(normalize_pane_content(""), "");
        assert_eq!(normalize_pane_content("\n\n\n"), "");
    }

    #[test]
    fn normalize_whitespace_only_pane() {
        assert_eq!(normalize_pane_content("   \n  \n   \n"), "");
    }

    #[test]
    fn normalize_preserves_content_lines() {
        let raw = "line1\nline2\nline3";
        assert_eq!(normalize_pane_content(raw), "line1\nline2\nline3");
    }

    #[test]
    fn normalize_preserves_internal_blank_lines() {
        let raw = "line1\n\nline3\n\n\n";
        assert_eq!(normalize_pane_content(raw), "line1\n\nline3");
    }

    #[test]
    fn normalize_handles_binary_replacement_chars() {
        let raw = "hello\u{FFFD}world\n\n\n";
        assert_eq!(normalize_pane_content(raw), "hello\u{FFFD}world");
    }

    #[test]
    fn captured_content_type() {
        let content = CapturedContent::new("hello  \n\n\n");
        assert_eq!(content.as_str(), "hello");
        assert_eq!(String::from(content.clone()), "hello");
        assert_eq!(format!("{}", content), "hello");
    }
}
