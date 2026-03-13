use tmux_interface::{KillWindow as TmuxKillWindow, Tmux};

pub async fn kill_window(target: &str) -> Result<(), String> {
    let target = target.to_string();
    tokio::task::spawn_blocking(move || {
        let output = Tmux::with_command(TmuxKillWindow::new().target_window(target.as_str()))
            .output()
            .map_err(|e| format!("failed to run tmux: {e}"))?
            .into_inner();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tmux kill-window failed: {stderr}"));
        }

        Ok(())
    })
    .await
    .map_err(|e| format!("task join error: {e}"))?
}
