use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub created: String,
    pub attached: bool,
}

pub async fn list_sessions() -> Result<Vec<TmuxSession>, String> {
    let output = Command::new("tmux")
        .args(["ls", "-F", "#{session_name}\t#{session_windows}\t#{session_created_string}\t#{session_attached}"])
        .output()
        .await
        .map_err(|e| format!("failed to run tmux: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") || stderr.contains("no sessions") {
            return Ok(vec![]);
        }
        return Err(format!("tmux ls failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() < 4 {
                return None;
            }
            Some(TmuxSession {
                name: parts[0].to_string(),
                windows: parts[1].parse().unwrap_or(0),
                created: parts[2].to_string(),
                attached: parts[3] == "1",
            })
        })
        .collect();

    Ok(sessions)
}
