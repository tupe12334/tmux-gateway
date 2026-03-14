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
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 4 {
                    return None;
                }
                Some(TmuxPane {
                    id: parts[0].to_string(),
                    width: parts[1].parse().unwrap_or(0),
                    height: parts[2].parse().unwrap_or(0),
                    active: parts[3] == "1",
                })
            })
            .collect();

        Ok(panes)
    })
    .await
    .map_err(|e| TmuxError::CommandFailed {
        command: "list-panes".to_string(),
        stderr: format!("task join error: {e}"),
    })?
}
