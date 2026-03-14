use std::fmt;

use super::TmuxError;
use super::validation::validate_session_target;
use crate::executor::TmuxExecutor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxWindow {
    pub id: String,
    pub index: u32,
    pub name: String,
    pub panes: u32,
    pub active: bool,
}

impl fmt::Display for TmuxWindow {
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

pub(crate) fn parse_window_line(line: &str) -> Result<TmuxWindow, TmuxError> {
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

pub(crate) fn parse_windows(stdout: &str) -> Result<Vec<TmuxWindow>, TmuxError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_window_line)
        .collect()
}

#[tracing::instrument(skip(executor))]
pub async fn list_windows(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &str,
) -> Result<Vec<TmuxWindow>, TmuxError> {
    validate_session_target(session)?;
    let output = executor
        .execute(&[
            "list-windows",
            "-t",
            session,
            "-F",
            "#{window_id}\t#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}",
        ])
        .await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "list-windows",
            &output.stderr,
            session,
        ));
    }
    parse_windows(&output.stdout)
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
    fn parse_windows_multiple_lines() {
        let input = "@0\t0\tbash\t1\t1\n@1\t1\tvim\t2\t0\n";
        let windows = parse_windows(input).unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].id, "@0");
        assert_eq!(windows[0].name, "bash");
        assert_eq!(windows[1].id, "@1");
        assert_eq!(windows[1].name, "vim");
    }

    #[test]
    fn parse_windows_empty_input() {
        let windows = parse_windows("").unwrap();
        assert!(windows.is_empty());
    }

    #[test]
    fn parse_windows_skips_empty_lines() {
        let input = "\n@0\t0\tbash\t1\t1\n\n";
        let windows = parse_windows(input).unwrap();
        assert_eq!(windows.len(), 1);
    }

    #[test]
    fn parse_windows_propagates_error() {
        let input = "@0\t0\tbash\t1\t1\nbad line";
        let result = parse_windows(input);
        assert!(result.is_err());
    }
}
