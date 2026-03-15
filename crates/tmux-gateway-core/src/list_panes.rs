use std::fmt;

use super::TmuxError;
use super::validation::validate_window_target;
use crate::executor::TmuxExecutor;
use crate::pagination::{PaginatedResult, Pagination, paginate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxPane {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
    pub current_path: String,
    pub current_command: String,
    pub pid: u32,
}

impl fmt::Display for TmuxPane {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}x{}{}, {})",
            self.id,
            self.width,
            self.height,
            if self.active { ", active" } else { "" },
            self.current_command
        )
    }
}

pub(crate) fn parse_pane_line(line: &str) -> Result<TmuxPane, TmuxError> {
    let parts: Vec<&str> = line.splitn(7, '\t').collect();
    if parts.len() < 7 {
        return Err(TmuxError::ParseError {
            command: "list-panes".to_string(),
            details: format!("expected 7 tab-separated fields, got: {line}"),
        });
    }
    let width = parts[1].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-panes".to_string(),
        details: format!("invalid width '{}': {e}", parts[1]),
    })?;
    let height = parts[2].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-panes".to_string(),
        details: format!("invalid height '{}': {e}", parts[2]),
    })?;
    let pid = parts[6].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-panes".to_string(),
        details: format!("invalid pid '{}': {e}", parts[6]),
    })?;
    Ok(TmuxPane {
        id: parts[0].to_string(),
        width,
        height,
        active: parts[3] == "1",
        current_path: parts[4].to_string(),
        current_command: parts[5].to_string(),
        pid,
    })
}

#[tracing::instrument(skip(executor))]
pub async fn list_panes(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<Vec<TmuxPane>, TmuxError> {
    validate_window_target(target)?;
    let output = executor
        .execute(&[
            "list-panes",
            "-t",
            target,
            "-F",
            "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}\t#{pane_pid}",
        ])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr("list-panes", &output.stderr, target));
    }
    output
        .stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_pane_line)
        .collect()
}

#[tracing::instrument(skip(executor))]
pub async fn list_panes_paginated(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    pagination: &Pagination,
) -> Result<PaginatedResult<TmuxPane>, TmuxError> {
    let all = list_panes(executor, target).await?;
    Ok(paginate(all, pagination))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_active_pane() {
        let pane = TmuxPane {
            id: "%0".to_string(),
            width: 80,
            height: 24,
            active: true,
            current_path: "/home/user".to_string(),
            current_command: "bash".to_string(),
            pid: 1234,
        };
        assert_eq!(pane.to_string(), "%0 (80x24, active, bash)");
    }

    #[test]
    fn display_inactive_pane() {
        let pane = TmuxPane {
            id: "%1".to_string(),
            width: 120,
            height: 40,
            active: false,
            current_path: "/tmp".to_string(),
            current_command: "vim".to_string(),
            pid: 5678,
        };
        assert_eq!(pane.to_string(), "%1 (120x40, vim)");
    }

    #[test]
    fn parse_pane_line_valid() {
        let pane = parse_pane_line("%0\t80\t24\t1\t/home/user\tbash\t1234").unwrap();
        assert_eq!(pane.id, "%0");
        assert_eq!(pane.width, 80);
        assert_eq!(pane.height, 24);
        assert!(pane.active);
        assert_eq!(pane.current_path, "/home/user");
        assert_eq!(pane.current_command, "bash");
        assert_eq!(pane.pid, 1234);
    }

    #[test]
    fn parse_pane_line_inactive() {
        let pane = parse_pane_line("%1\t120\t40\t0\t/tmp\tvim\t5678").unwrap();
        assert!(!pane.active);
        assert_eq!(pane.current_path, "/tmp");
        assert_eq!(pane.current_command, "vim");
        assert_eq!(pane.pid, 5678);
    }

    #[test]
    fn parse_pane_line_missing_fields() {
        let result = parse_pane_line("%0\t80");
        assert!(result.is_err());
    }

    #[test]
    fn parse_pane_line_invalid_width() {
        let result = parse_pane_line("%0\tabc\t24\t1\t/home\tbash\t1234");
        assert!(result.is_err());
    }

    #[test]
    fn parse_pane_line_invalid_height() {
        let result = parse_pane_line("%0\t80\txyz\t1\t/home\tbash\t1234");
        assert!(result.is_err());
    }

    #[test]
    fn parse_pane_line_invalid_pid() {
        let result = parse_pane_line("%0\t80\t24\t1\t/home\tbash\tnotanumber");
        assert!(result.is_err());
    }

    fn parse_panes(stdout: &str) -> Result<Vec<TmuxPane>, TmuxError> {
        stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(parse_pane_line)
            .collect()
    }

    #[test]
    fn parse_panes_multiple_lines() {
        let input = "%0\t80\t24\t1\t/home\tbash\t1234\n%1\t80\t24\t0\t/tmp\tzsh\t5678\n";
        let panes = parse_panes(input).unwrap();
        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].id, "%0");
        assert_eq!(panes[0].pid, 1234);
        assert_eq!(panes[1].id, "%1");
        assert_eq!(panes[1].pid, 5678);
    }

    #[test]
    fn parse_panes_empty_input() {
        let panes = parse_panes("").unwrap();
        assert!(panes.is_empty());
    }

    #[test]
    fn parse_panes_skips_empty_lines() {
        let input = "\n%0\t80\t24\t1\t/home\tbash\t1234\n\n";
        let panes = parse_panes(input).unwrap();
        assert_eq!(panes.len(), 1);
    }

    #[test]
    fn parse_panes_propagates_error() {
        let input = "%0\t80\t24\t1\t/home\tbash\t1234\nbad line";
        let result = parse_panes(input);
        assert!(result.is_err());
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

    fn mock_panes_executor(count: usize) -> MockExecutor {
        let stdout = (0..count)
            .map(|i| format!("%{i}\t80\t24\t0\t/home\tbash\t{}", 1000 + i))
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
    async fn list_panes_paginated_default_returns_all() {
        let executor = mock_panes_executor(4);
        let result = list_panes_paginated(&executor, "sess:win", &Pagination::default())
            .await
            .unwrap();
        assert_eq!(result.items.len(), 4);
        assert_eq!(result.total, 4);
        assert!(!result.has_more);
    }

    #[tokio::test]
    async fn list_panes_paginated_with_limit() {
        let executor = mock_panes_executor(4);
        let result = list_panes_paginated(
            &executor,
            "sess:win",
            &Pagination {
                offset: 0,
                limit: Some(2),
            },
        )
        .await
        .unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.items[0].id, "%0");
        assert_eq!(result.items[1].id, "%1");
        assert_eq!(result.total, 4);
        assert!(result.has_more);
    }

    #[tokio::test]
    async fn list_panes_paginated_offset_beyond_total() {
        let executor = mock_panes_executor(3);
        let result = list_panes_paginated(
            &executor,
            "sess:win",
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
