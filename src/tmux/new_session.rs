use tokio::process::Command;

pub async fn new_session(name: &str) -> Result<String, String> {
    let output = Command::new("tmux")
        .args(["new-session", "-d", "-s", name])
        .output()
        .await
        .map_err(|e| format!("failed to run tmux: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux new-session failed: {stderr}"));
    }

    Ok(name.to_string())
}
