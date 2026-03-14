use crate::executor::TmuxExecutor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxServerInfo {
    pub version: String,
    pub running: bool,
}

/// Returns tmux server info (version and running status).
pub async fn server_info(executor: &(impl TmuxExecutor + ?Sized)) -> TmuxServerInfo {
    match executor.execute(&["-V"]).await {
        Ok(output) => {
            if output.success {
                TmuxServerInfo {
                    version: output.stdout.trim().to_string(),
                    running: true,
                }
            } else {
                TmuxServerInfo {
                    version: String::new(),
                    running: false,
                }
            }
        }
        Err(_) => TmuxServerInfo {
            version: String::new(),
            running: false,
        },
    }
}

/// Returns `true` if tmux is reachable and responding.
pub async fn is_available(executor: &(impl TmuxExecutor + ?Sized)) -> bool {
    server_info(executor).await.running
}
