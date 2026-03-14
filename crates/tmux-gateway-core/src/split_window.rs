use tmux_interface::{SplitWindow, Tmux};

use super::TmuxError;
use super::validation::validate_pane_target;
use crate::TmuxPane;

pub async fn split_window(target: &str, horizontal: bool) -> Result<TmuxPane, TmuxError> {
    validate_pane_target(target)?;
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let mut cmd = SplitWindow::new()
            .detached()
            .target_pane(target.as_str())
            .print()
            .format("#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}");
        if horizontal {
            cmd = cmd.horizontal();
        } else {
            cmd = cmd.vertical();
        }

        let output = Tmux::with_command(cmd)
            .output()
            .map_err(|e| TmuxError::CommandFailed {
                command: "split-window".to_string(),
                stderr: e.to_string(),
            })?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("split-window", &stderr, &target));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() < 4 {
            return Err(TmuxError::ParseError {
                command: "split-window".to_string(),
                details: format!("expected 4 tab-separated fields, got: {line}"),
            });
        }
        let width = parts[1].parse::<u32>().map_err(|e| TmuxError::ParseError {
            command: "split-window".to_string(),
            details: format!("invalid width '{}': {e}", parts[1]),
        })?;
        let height = parts[2].parse::<u32>().map_err(|e| TmuxError::ParseError {
            command: "split-window".to_string(),
            details: format!("invalid height '{}': {e}", parts[2]),
        })?;
        Ok(TmuxPane {
            id: parts[0].to_string(),
            width,
            height,
            active: parts[3] == "1",
        })
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "split-window".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
