use tokio::process::Command;

pub async fn kill_window(target: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .args(["kill-window", "-t", target])
        .output()
        .await
        .map_err(|e| format!("failed to run tmux: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux kill-window failed: {stderr}"));
    }

    Ok(())
}
