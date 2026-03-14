use serde::Serialize;
use tmux_interface::{ListPanes, Tmux};

use super::TmuxError;
use super::validation::validate_window_target;

#[derive(Debug, Clone, Serialize)]
pub struct TmuxPane {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
}

pub async fn list_panes(target: &str) -> Result<Vec<TmuxPane>, TmuxError> {
    validate_window_target(target)?;
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(
            ListPanes::new()
                .target(target.as_str())
                .format("#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_active}"),
        )
        .output()
        .map_err(|e| TmuxError::CommandFailed {
            command: "list-panes".to_string(),
            stderr: e.to_string(),
        })?
        .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::from_stderr("list-panes", &stderr, &target));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let panes = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 4 {
                    return Err(TmuxError::ParseError {
                        command: "list-panes".to_string(),
                        details: format!("expected 4 tab-separated fields, got: {line}"),
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
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(panes)
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "list-panes".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
