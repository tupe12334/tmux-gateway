use tokio::process::Command;

pub async fn kill_session(target: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(["kill-session", "-t", target])
        .output()
        .await
        .map_err(|e| format!("failed to run tmux: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux kill-session failed: {stderr}"));
    }

    Ok(())
}
