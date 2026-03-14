use serde::Serialize;
use tmux_interface::Tmux;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TmuxServerInfo {
    pub version: String,
    pub running: bool,
}

/// Returns tmux server info (version and running status).
///
/// This is a blocking call wrapped in `spawn_blocking` for async contexts.
pub async fn server_info() -> TmuxServerInfo {
    tokio::task::spawn_blocking(server_info_blocking)
        .await
        .unwrap_or(TmuxServerInfo {
            version: String::new(),
            running: false,
        })
}

/// Returns `true` if tmux is reachable and responding.
pub async fn is_available() -> bool {
    server_info().await.running
}

/// Synchronous version of [`server_info`] for use outside async contexts.
pub fn server_info_blocking() -> TmuxServerInfo {
    match Tmux::new().version().output() {
        Ok(output) => {
            let raw = output.into_inner();
            if raw.status.success() {
                let version = String::from_utf8_lossy(&raw.stdout).trim().to_string();
                TmuxServerInfo {
                    version,
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
