use std::fmt;

use super::TmuxError;
use super::validation::SessionName;
use crate::command_spec::TmuxCommandSpec;
use crate::executor::TmuxExecutor;
use crate::pagination::{PaginatedResult, Pagination, paginate};

/// A tmux window.
///
/// [tmux docs](https://man.openbsd.org/tmux#WINDOWS_AND_PANES)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxWindow {
    pub id: String,
    pub index: u32,
    pub name: String,
    pub panes: u32,
    pub active: bool,
}

impl fmt::Display for TmuxWindow {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} ({} panes{})",
            self.index,
            self.name,
            self.panes,
            if self.active { ", active" } else { "" }
        )
    }
}

/// Pure: build the tmux command specification for listing windows.
pub fn build_list_windows_command(session: &SessionName) -> TmuxCommandSpec {
    TmuxCommandSpec::new(vec![
        "list-windows".into(),
        "-t".into(),
        session.as_str().into(),
        "-F".into(),
        "#{window_id}\t#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}".into(),
    ])
}

/// Pure: parse a single line of list-windows output into a TmuxWindow.
pub fn parse_window_line(line: &str) -> Result<TmuxWindow, TmuxError> {
    let parts: Vec<&str> = line.splitn(5, '\t').collect();
    if parts.len() < 5 {
        return Err(TmuxError::ParseError {
            command: "list-windows".to_string(),
            details: format!("expected 5 tab-separated fields, got: {line}"),
        });
    }
    let index = parts[1].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-windows".to_string(),
        details: format!("invalid window index '{}': {e}", parts[1]),
    })?;
    let panes = parts[3].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-windows".to_string(),
        details: format!("invalid pane count '{}': {e}", parts[3]),
    })?;
    Ok(TmuxWindow {
        id: parts[0].to_string(),
        index,
        name: parts[2].to_string(),
        panes,
        active: parts[4] == "1",
    })
}

/// Pure: parse raw list-windows stdout into domain types.
pub fn parse_list_windows_output(stdout: &str) -> Result<Vec<TmuxWindow>, TmuxError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_window_line)
        .collect()
}

/// List all windows in a session.
///
/// [tmux docs](https://man.openbsd.org/tmux#list-windows)
#[tracing::instrument(skip(executor))]
pub async fn list_windows(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &SessionName,
) -> Result<Vec<TmuxWindow>, TmuxError> {
    let spec = build_list_windows_command(session);
    let output = executor.execute(&spec.args()).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            spec.command_name(),
            &output.stderr,
            session.as_str(),
        ));
    }
    parse_list_windows_output(&output.stdout)
}

#[tracing::instrument(skip(executor))]
pub async fn list_windows_paginated(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &SessionName,
    pagination: &Pagination,
) -> Result<PaginatedResult<TmuxWindow>, TmuxError> {
    let all = list_windows(executor, session).await?;
    Ok(paginate(all, pagination))
}

#[tracing::instrument(skip(executor))]
pub async fn get_window(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &SessionName,
    name: &str,
) -> Result<Option<TmuxWindow>, TmuxError> {
    let windows = list_windows(executor, session).await?;
    Ok(windows.into_iter().find(|w| w.name == name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_active_window() {
        let window = TmuxWindow {
            id: "@0".to_string(),
            index: 0,
            name: "bash".to_string(),
            panes: 2,
            active: true,
        };
        assert_eq!(window.to_string(), "0:bash (2 panes, active)");
    }

    #[test]
    fn display_inactive_window() {
        let window = TmuxWindow {
            id: "@1".to_string(),
            index: 1,
            name: "vim".to_string(),
            panes: 1,
            active: false,
        };
        assert_eq!(window.to_string(), "1:vim (1 panes)");
    }

    #[test]
    fn parse_window_line_valid() {
        let window = parse_window_line("@0\t0\tbash\t1\t1").unwrap();
        assert_eq!(window.id, "@0");
        assert_eq!(window.index, 0);
        assert_eq!(window.name, "bash");
        assert_eq!(window.panes, 1);
        assert!(window.active);
    }

    #[test]
    fn parse_window_line_inactive() {
        let window = parse_window_line("@2\t2\tvim\t3\t0").unwrap();
        assert_eq!(window.id, "@2");
        assert!(!window.active);
        assert_eq!(window.panes, 3);
    }

    #[test]
    fn parse_window_line_missing_fields() {
        let result = parse_window_line("@0\t0\tbash");
        assert!(result.is_err());
    }

    #[test]
    fn parse_window_line_invalid_index() {
        let result = parse_window_line("@0\tabc\tbash\t1\t1");
        assert!(result.is_err());
    }

    #[test]
    fn parse_window_line_invalid_pane_count() {
        let result = parse_window_line("@0\t0\tbash\txyz\t1");
        assert!(result.is_err());
    }

    #[test]
    fn parse_list_windows_output_multiple_lines() {
        let input = "@0\t0\tbash\t1\t1\n@1\t1\tvim\t2\t0\n";
        let windows = parse_list_windows_output(input).unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].id, "@0");
        assert_eq!(windows[0].name, "bash");
        assert_eq!(windows[1].id, "@1");
        assert_eq!(windows[1].name, "vim");
    }

    #[test]
    fn parse_list_windows_output_empty_input() {
        let windows = parse_list_windows_output("").unwrap();
        assert!(windows.is_empty());
    }

    #[test]
    fn parse_list_windows_output_skips_empty_lines() {
        let input = "\n@0\t0\tbash\t1\t1\n\n";
        let windows = parse_list_windows_output(input).unwrap();
        assert_eq!(windows.len(), 1);
    }

    #[test]
    fn parse_list_windows_output_propagates_error() {
        let input = "@0\t0\tbash\t1\t1\nbad line";
        let result = parse_list_windows_output(input);
        assert!(result.is_err());
    }

    #[test]
    fn build_list_windows_command_produces_correct_args() {
        let session = SessionName::try_from("my-session").unwrap();
        let spec = build_list_windows_command(&session);
        assert_eq!(spec.command_name(), "list-windows");
        let args = spec.args();
        assert_eq!(args[1], "-t");
        assert_eq!(args[2], "my-session");
        assert_eq!(args[3], "-F");
        assert!(args[4].contains("window_id"));
    }

    // ── Mock executor tests for pagination ──

    use crate::executor::TmuxOutput;

    struct MockExecutor {
        output: TmuxOutput,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            Ok(self.output.clone())
        }
    }

    fn mock_windows_executor(count: usize) -> MockExecutor {
        let stdout = (0..count)
            .map(|i| format!("@{i}\t{i}\twin{i}\t1\t0"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        MockExecutor {
            output: TmuxOutput {
                stdout,
                stderr: String::new(),
                success: true,
            },
        }
    }

    #[tokio::test]
    async fn list_windows_paginated_default_returns_all() {
        let executor = mock_windows_executor(4);
        let session = SessionName::try_from("test").unwrap();
        let result = list_windows_paginated(&executor, &session, &Pagination::default())
            .await
            .unwrap();
        assert_eq!(result.items.len(), 4);
        assert_eq!(result.total, 4);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn list_windows_paginated_with_limit() {
        let executor = mock_windows_executor(4);
        let session = SessionName::try_from("test").unwrap();
        let result = list_windows_paginated(
            &executor,
            &session,
            &Pagination {
                offset: 0,
                limit: Some(2),
            },
        )
        .await
        .unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].name, "win0");
        assert_eq!(result.items[1].name, "win1");
        assert_eq!(result.total, 4);
        assert!(result.has_more);
    }

    #[tokio::test]
    async fn list_windows_paginated_offset_beyond_total() {
        let executor = mock_windows_executor(3);
        let session = SessionName::try_from("test").unwrap();
        let result = list_windows_paginated(
            &executor,
            &session,
            &Pagination {
                offset: 50,
                limit: Some(10),
            },
        )
        .await
        .unwrap();
        assert!(result.items.is_empty());
        assert_eq!(result.total, 3);
        assert!(!result.has_more);
    }
}
