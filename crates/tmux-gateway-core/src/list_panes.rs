use std::fmt;

use super::TmuxError;
use super::validation::validate_window_target;
use crate::executor::TmuxExecutor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxPane {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
    pub current_path: String,
    pub current_command: String,
}

impl fmt::Display for TmuxPane {
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
    let parts: Vec<&str> = line.splitn(6, '\t').collect();
    if parts.len() < 6 {
        return Err(TmuxError::ParseError {
            command: "list-panes".to_string(),
            details: format!("expected 6 tab-separated fields, got: {line}"),
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
    Ok(TmuxPane {
        id: parts[0].to_string(),
        width,
        height,
        active: parts[3] == "1",
        current_path: parts[4].to_string(),
        current_command: parts[5].to_string(),
    })
}

pub(crate) fn parse_panes(stdout: &str) -> Result<Vec<TmuxPane>, TmuxError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_pane_line)
        .collect()
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
            "#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}",
        ])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr("list-panes", &output.stderr, target));
    }
    parse_panes(&output.stdout)
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
        };
        assert_eq!(pane.to_string(), "%1 (120x40, vim)");
    }

    #[test]
    fn parse_pane_line_valid() {
        let pane = parse_pane_line("%0\t80\t24\t1\t/home/user\tbash").unwrap();
        assert_eq!(pane.id, "%0");
        assert_eq!(pane.width, 80);
        assert_eq!(pane.height, 24);
        assert!(pane.active);
        assert_eq!(pane.current_path, "/home/user");
        assert_eq!(pane.current_command, "bash");
    }

    #[test]
    fn parse_pane_line_inactive() {
        let pane = parse_pane_line("%1\t120\t40\t0\t/tmp\tvim").unwrap();
        assert!(!pane.active);
        assert_eq!(pane.current_path, "/tmp");
        assert_eq!(pane.current_command, "vim");
    }

    #[test]
    fn parse_pane_line_missing_fields() {
        let result = parse_pane_line("%0\t80");
        assert!(result.is_err());
    }

    #[test]
    fn parse_pane_line_invalid_width() {
        let result = parse_pane_line("%0\tabc\t24\t1\t/home\tbash");
        assert!(result.is_err());
    }

    #[test]
    fn parse_pane_line_invalid_height() {
        let result = parse_pane_line("%0\t80\txyz\t1\t/home\tbash");
        assert!(result.is_err());
    }

    #[test]
    fn parse_panes_multiple_lines() {
        let input = "%0\t80\t24\t1\t/home\tbash\n%1\t80\t24\t0\t/tmp\tzsh\n";
        let panes = parse_panes(input).unwrap();
        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].id, "%0");
        assert_eq!(panes[1].id, "%1");
    }

    #[test]
    fn parse_panes_empty_input() {
        let panes = parse_panes("").unwrap();
        assert!(panes.is_empty());
    }

    #[test]
    fn parse_panes_skips_empty_lines() {
        let input = "\n%0\t80\t24\t1\t/home\tbash\n\n";
        let panes = parse_panes(input).unwrap();
        assert_eq!(panes.len(), 1);
    }

    #[test]
    fn parse_panes_propagates_error() {
        let input = "%0\t80\t24\t1\t/home\tbash\nbad line";
        let result = parse_panes(input);
        assert!(result.is_err());
    }
}
