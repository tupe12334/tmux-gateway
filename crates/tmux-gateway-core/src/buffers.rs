use std::fmt;

use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::validation::{PaneTarget, validate_buffer_name};

use super::TmuxError;

/// A tmux paste buffer.
///
/// [tmux docs](https://man.openbsd.org/tmux#BUFFERS)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxBuffer {
    pub name: String,
    pub size: u64,
}

impl fmt::Display for TmuxBuffer {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} bytes)", self.name, self.size)
    }
}

/// Pure: build the tmux command specification for listing buffers.
pub fn build_list_buffers_command() -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "list-buffers".into(),
        "-F".into(),
        "#{buffer_name}\t#{buffer_size}".into(),
    ])
}

/// Pure: build the tmux command specification for showing a buffer.
pub fn build_get_buffer_command(name: Option<&str>) -> TmuxCommandSpec {
    let mut args = vec!["show-buffer".to_string()];
    if let Some(n) = name {
        args.push("-b".into());
        args.push(n.into());
    }
    TmuxCommandSpec::new(args)
}

/// Pure: build the tmux command specification for setting a buffer.
pub fn build_set_buffer_command(name: Option<&str>, content: &str) -> TmuxCommandSpec {
    let mut args = vec!["set-buffer".to_string()];
    if let Some(n) = name {
        args.push("-b".into());
        args.push(n.into());
    }
    args.push(content.into());
    TmuxCommandSpec::new(args)
}

/// Pure: build the tmux command specification for pasting a buffer.
pub fn build_paste_buffer_command(target: &PaneTarget, name: Option<&str>) -> TmuxCommandSpec {
    let mut args = vec![
        "paste-buffer".to_string(),
        "-t".to_string(),
        target.as_str().to_string(),
    ];
    if let Some(n) = name {
        args.push("-b".into());
        args.push(n.into());
    }
    TmuxCommandSpec::new(args)
}

/// Pure: build the tmux command specification for deleting a buffer.
pub fn build_delete_buffer_command(name: &str) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec!["delete-buffer".into(), "-b".into(), name.into()])
}

/// Pure: parse a single line of list-buffers output into a TmuxBuffer.
pub fn parse_buffer_line(line: &str) -> Result<TmuxBuffer, TmuxError> {
    let parts: Vec<&str> = line.splitn(2, '\t').collect();
    if parts.len() < 2 {
        return Err(TmuxError::ParseError {
            command: "list-buffers".to_string(),
            details: format!("expected 2 tab-separated fields, got: {line}"),
        });
    }
    let size = parts[1].parse::<u64>().map_err(|e| TmuxError::ParseError {
        command: "list-buffers".to_string(),
        details: format!("invalid buffer size '{}': {e}", parts[1]),
    })?;
    Ok(TmuxBuffer {
        name: parts[0].to_string(),
        size,
    })
}

/// Pure: parse raw list-buffers stdout into domain types.
#[allow(unknown_lints, no_wrapper_functions)]
pub fn parse_list_buffers_output(stdout: &str) -> Result<Vec<TmuxBuffer>, TmuxError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_buffer_line)
        .collect()
}

/// List all paste buffers.
///
/// [tmux docs](https://man.openbsd.org/tmux#list-buffers)
#[tracing::instrument(skip(executor))]
pub async fn list_buffers(
    executor: &(impl TmuxExecutor + ?Sized),
) -> Result<Vec<TmuxBuffer>, TmuxError> {
    let spec = build_list_buffers_command();
    let output = executor.execute(&spec.args()).await?;

    if !output.success {
        let stderr = &output.stderr;
        if stderr.contains("no buffers") {
            return Ok(vec![]);
        }
        return Err(TmuxError::from_stderr(spec.command_name(), stderr, ""));
    }

    parse_list_buffers_output(&output.stdout)
}

/// Get the content of a paste buffer.
/// If `name` is `None`, gets the most recent buffer.
///
/// [tmux docs](https://man.openbsd.org/tmux#show-buffer)
#[tracing::instrument(skip(executor))]
pub async fn get_buffer(
    executor: &(impl TmuxExecutor + ?Sized),
    name: Option<&str>,
) -> Result<String, TmuxError> {
    if let Some(n) = name {
        validate_buffer_name(n)?;
    }
    let spec = build_get_buffer_command(name);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            name.unwrap_or(""),
        ));
    }
    Ok(output.stdout)
}

/// Set the content of a paste buffer.
/// If `name` is `None`, creates a new buffer with an auto-assigned name.
///
/// [tmux docs](https://man.openbsd.org/tmux#set-buffer)
#[tracing::instrument(skip(executor, content))]
pub async fn set_buffer(
    executor: &(impl TmuxExecutor + ?Sized),
    name: Option<&str>,
    content: &str,
) -> Result<(), TmuxError> {
    if let Some(n) = name {
        validate_buffer_name(n)?;
    }
    let spec = build_set_buffer_command(name, content);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            name.unwrap_or(""),
        ));
    }
    Ok(())
}

/// Paste buffer content into a pane.
/// If `name` is `None`, pastes the most recent buffer.
///
/// [tmux docs](https://man.openbsd.org/tmux#paste-buffer)
#[tracing::instrument(skip(executor))]
pub async fn paste_buffer(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &PaneTarget,
    name: Option<&str>,
) -> Result<(), TmuxError> {
    if let Some(n) = name {
        validate_buffer_name(n)?;
    }
    let spec = build_paste_buffer_command(target, name);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            target.as_str(),
        ));
    }
    Ok(())
}

/// Delete a paste buffer.
///
/// [tmux docs](https://man.openbsd.org/tmux#delete-buffer)
#[tracing::instrument(skip(executor))]
pub async fn delete_buffer(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<(), TmuxError> {
    validate_buffer_name(name)?;
    let spec = build_delete_buffer_command(name);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            name,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    // ── Display ──

    #[test]
    fn display_buffer() {
        let buf = TmuxBuffer {
            name: "buffer0".to_string(),
            size: 42,
        };
        assert_eq!(buf.to_string(), "buffer0 (42 bytes)");
    }

    // ── Parsing ──

    #[test]
    fn parse_buffer_line_valid() {
        let buf = parse_buffer_line("buffer0\t128").unwrap();
        assert_eq!(buf.name, "buffer0");
        assert_eq!(buf.size, 128);
    }

    #[test]
    fn parse_buffer_line_missing_fields() {
        let result = parse_buffer_line("onlyname");
        assert!(result.is_err());
    }

    #[test]
    fn parse_buffer_line_invalid_size() {
        let result = parse_buffer_line("buffer0\tnotanum");
        assert!(result.is_err());
    }

    #[test]
    fn parse_buffer_line_zero_size() {
        let buf = parse_buffer_line("buf\t0").unwrap();
        assert_eq!(buf.size, 0);
    }

    #[test]
    fn parse_buffers_multiple_lines() {
        let input = "buffer0\t100\nbuffer1\t200\n";
        let buffers: Vec<TmuxBuffer> = input
            .lines()
            .filter(|l| !l.is_empty())
            .map(parse_buffer_line)
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(buffers.len(), 2);
        assert_eq!(buffers[0].name, "buffer0");
        assert_eq!(buffers[1].name, "buffer1");
    }

    #[test]
    fn parse_buffers_empty_input() {
        let buffers: Vec<TmuxBuffer> = ""
            .lines()
            .filter(|l| !l.is_empty())
            .map(parse_buffer_line)
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(buffers.is_empty());
    }

    // ── Mock executor tests ──

    struct MockExecutor {
        output: TmuxOutput,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            Ok(self.output.clone())
        }
    }

    #[tokio::test]
    async fn list_buffers_parses_mock_output() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "buffer0\t100\nbuffer1\t200\n".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let buffers = list_buffers(&executor).await.unwrap();
        assert_eq!(buffers.len(), 2);
        assert_eq!(buffers[0].name, "buffer0");
        assert_eq!(buffers[0].size, 100);
        assert_eq!(buffers[1].name, "buffer1");
        assert_eq!(buffers[1].size, 200);
    }

    #[tokio::test]
    async fn list_buffers_empty_on_no_buffers() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no buffers".to_string(),
                success: false,
            },
        };
        let buffers = list_buffers(&executor).await.unwrap();
        assert!(buffers.is_empty());
    }

    #[tokio::test]
    async fn list_buffers_error_on_failure() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux".to_string(),
                success: false,
            },
        };
        let result = list_buffers(&executor).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    #[tokio::test]
    async fn get_buffer_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "hello world".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let content = get_buffer(&executor, Some("buffer0")).await.unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn get_buffer_no_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "content".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let content = get_buffer(&executor, None).await.unwrap();
        assert_eq!(content, "content");
    }

    #[tokio::test]
    async fn get_buffer_invalid_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = get_buffer(&executor, Some("buf;injection")).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_buffer_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = set_buffer(&executor, Some("mybuf"), "data").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn set_buffer_no_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = set_buffer(&executor, None, "data").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn set_buffer_invalid_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = set_buffer(&executor, Some("$(evil)"), "data").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn paste_buffer_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = paste_buffer(&executor, &target, Some("buffer0")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn paste_buffer_invalid_target() {
        // With newtypes, invalid targets are caught at construction time
        assert!(PaneTarget::try_from("bad-target").is_err());
    }

    #[tokio::test]
    async fn paste_buffer_invalid_buffer_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let target = PaneTarget::try_from("sess:0.0").unwrap();
        let result = paste_buffer(&executor, &target, Some("buf;evil")).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_buffer_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = delete_buffer(&executor, "buffer0").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_buffer_invalid_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = delete_buffer(&executor, "").await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_buffer_failure() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux".to_string(),
                success: false,
            },
        };
        let result = delete_buffer(&executor, "buffer0").await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
