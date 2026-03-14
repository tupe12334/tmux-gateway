use crate::TmuxError;

#[derive(Debug, Clone)]
pub struct TmuxOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub trait TmuxExecutor: Send + Sync {
    fn execute(
        &self,
        args: &[&str],
    ) -> impl std::future::Future<Output = Result<TmuxOutput, TmuxError>> + Send;
}

#[derive(Debug, Clone, Copy)]
pub struct RealTmuxExecutor;

impl TmuxExecutor for RealTmuxExecutor {
    async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let cmd_name = args.first().cloned().unwrap_or_default();
        tokio::task::spawn_blocking(move || {
            let output = std::process::Command::new("tmux")
                .args(&args)
                .output()
                .map_err(|e| TmuxError::CommandFailed {
                    command: cmd_name.clone(),
                    stderr: e.to_string(),
                })?;
            Ok(TmuxOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                success: output.status.success(),
            })
        })
        .await
        .map_err(|e| TmuxError::CommandFailed {
            command: "spawn_blocking".to_string(),
            stderr: format!("task join error: {e}"),
        })?
    }
}
