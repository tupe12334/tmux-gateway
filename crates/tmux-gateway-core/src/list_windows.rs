use serde::Serialize;
use tmux_interface::ListWindows;

use super::TmuxError;
use super::validation::validate_session_target;
use crate::executor::{exec_tmux, run_tmux};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TmuxWindow {
    pub index: u32,
    pub name: String,
    pub panes: u32,
    pub active: bool,
}

pub(crate) fn parse_window_line(line: &str) -> Result<TmuxWindow, TmuxError> {
    let parts: Vec<&str> = line.splitn(4, '\t').collect();
    if parts.len() < 4 {
        return Err(TmuxError::ParseError {
            command: "list-windows".to_string(),
            details: format!("expected 4 tab-separated fields, got: {line}"),
        });
    }
    let index = parts[0].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-windows".to_string(),
        details: format!("invalid window index '{}': {e}", parts[0]),
    })?;
    let panes = parts[2].parse::<u32>().map_err(|e| TmuxError::ParseError {
        command: "list-windows".to_string(),
        details: format!("invalid pane count '{}': {e}", parts[2]),
    })?;
    Ok(TmuxWindow {
        index,
        name: parts[1].to_string(),
        panes,
        active: parts[3] == "1",
    })
}

pub(crate) fn parse_windows(stdout: &str) -> Result<Vec<TmuxWindow>, TmuxError> {
    stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_window_line)
        .collect()
}

pub async fn list_windows(session: &str) -> Result<Vec<TmuxWindow>, TmuxError> {
    validate_session_target(session)?;
    let session = session.to_string();
    run_tmux("list-windows", move || {
        let output = exec_tmux(
            "list-windows",
            &session,
            ListWindows::new()
                .target_session(session.as_str())
                .format("#{window_index}\t#{window_name}\t#{window_panes}\t#{window_active}"),
        )?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_windows(&stdout)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_window_line_valid() {
        let window = parse_window_line("0\tbash\t1\t1").unwrap();
        assert_eq!(window.index, 0);
        assert_eq!(window.name, "bash");
        assert_eq!(window.panes, 1);
        assert!(window.active);
    }

    #[test]
    fn parse_window_line_inactive() {
        let window = parse_window_line("2\tvim\t3\t0").unwrap();
        assert!(!window.active);
        assert_eq!(window.panes, 3);
    }

    #[test]
    fn parse_window_line_missing_fields() {
        let result = parse_window_line("0\tbash");
        assert!(result.is_err());
    }

    #[test]
    fn parse_window_line_invalid_index() {
        let result = parse_window_line("abc\tbash\t1\t1");
        assert!(result.is_err());
    }

    #[test]
    fn parse_window_line_invalid_pane_count() {
        let result = parse_window_line("0\tbash\txyz\t1");
        assert!(result.is_err());
    }

    #[test]
    fn parse_windows_multiple_lines() {
        let input = "0\tbash\t1\t1\n1\tvim\t2\t0\n";
        let windows = parse_windows(input).unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].name, "bash");
        assert_eq!(windows[1].name, "vim");
    }

    #[test]
    fn parse_windows_empty_input() {
        let windows = parse_windows("").unwrap();
        assert!(windows.is_empty());
    }

    #[test]
    fn parse_windows_skips_empty_lines() {
        let input = "\n0\tbash\t1\t1\n\n";
        let windows = parse_windows(input).unwrap();
        assert_eq!(windows.len(), 1);
    }

    #[test]
    fn parse_windows_propagates_error() {
        let input = "0\tbash\t1\t1\nbad line";
        let result = parse_windows(input);
        assert!(result.is_err());
    }
}
