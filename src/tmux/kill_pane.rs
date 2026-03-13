use tokio::process::Command;

pub async fn kill_pane(target: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(["kill-pane", "-t", target])
        .output()
        .await
        .map_err(|e| format!("failed to run tmux: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux kill-pane failed: {stderr}"));
    }

    Ok(())
}
