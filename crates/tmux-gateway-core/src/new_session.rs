use tmux_interface::{NewSession, Tmux};

pub async fn new_session(name: &str) -> Result<String, String> {
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(NewSession::new().detached().session_name(name.as_str()))
            .output()
            .map_err(|e| format!("failed to run tmux: {e}"))?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tmux new-session failed: {stderr}"));
        }

        Ok(name)
    })
    .await
    .map_err(|e| format!("task join error: {e}"))?
}
