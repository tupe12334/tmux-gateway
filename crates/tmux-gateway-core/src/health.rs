use std::fmt;

use crate::executor::TmuxExecutor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthStatus {
    pub available: bool,
    pub version: String,
    pub session_count: u32,
    pub client_count: u32,
    pub uptime_seconds: Option<u64>,
    pub server_pid: Option<u32>,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.available {
            write!(
                f,
                "healthy ({}, {} sessions, {} clients)",
                self.version, self.session_count, self.client_count
            )
        } else {
            write!(f, "unavailable")
        }
    }
}

impl HealthStatus {
    fn unavailable() -> Self {
        Self {
            available: false,
            version: String::new(),
            session_count: 0,
            client_count: 0,
            uptime_seconds: None,
            server_pid: None,
        }
    }
}

/// Comprehensive health check of the tmux server.
///
/// Gathers data from multiple tmux commands (`-V`, `list-sessions`, `list-clients`,
/// `display-message`) into a single domain model. Never fails — if tmux is
/// unreachable, returns `HealthStatus { available: false, .. }`.
#[tracing::instrument(skip(executor))]
pub async fn health_check(executor: &(impl TmuxExecutor + ?Sized)) -> HealthStatus {
    // 1. Check version / reachability
    let version_output = match executor.execute(&["-V"]).await {
        Ok(o) if o.success => o,
        _ => return HealthStatus::unavailable(),
    };
    let version = version_output.stdout.trim().to_string();

    // 2. Count sessions
    let session_count = match executor
        .execute(&["list-sessions", "-F", "#{session_name}"])
        .await
    {
        Ok(o) if o.success => o.stdout.lines().filter(|l| !l.is_empty()).count() as u32,
        _ => 0,
    };

    // 3. Count clients
    let client_count = match executor
        .execute(&["list-clients", "-F", "#{client_name}"])
        .await
    {
        Ok(o) if o.success => o.stdout.lines().filter(|l| !l.is_empty()).count() as u32,
        _ => 0,
    };

    // 4. Server PID and uptime via display-message
    let (server_pid, uptime_seconds) = match executor
        .execute(&["display-message", "-p", "#{pid}\t#{start_time}"])
        .await
    {
        Ok(o) if o.success => parse_server_info(&o.stdout),
        _ => (None, None),
    };

    HealthStatus {
        available: true,
        version,
        session_count,
        client_count,
        uptime_seconds,
        server_pid,
    }
}

fn parse_server_info(stdout: &str) -> (Option<u32>, Option<u64>) {
    let line = stdout.trim();
    let parts: Vec<&str> = line.splitn(2, '\t').collect();
    if parts.len() < 2 {
        return (None, None);
    }
    let pid = parts[0].parse::<u32>().ok();
    let start_time = parts[1].parse::<u64>().ok();
    let uptime = start_time.and_then(|st| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        now.checked_sub(st)
    });
    (pid, uptime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TmuxError;
    use crate::executor::TmuxOutput;

    struct MockExecutor {
        responses: Vec<TmuxOutput>,
    }

    impl MockExecutor {
        fn new(responses: Vec<TmuxOutput>) -> Self {
            Self { responses }
        }
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            let cmd = args.first().copied().unwrap_or("");
            let idx = match cmd {
                "-V" => 0,
                "list-sessions" => 1,
                "list-clients" => 2,
                "display-message" => 3,
                _ => return Err(TmuxError::TmuxNotRunning),
            };
            if idx < self.responses.len() {
                Ok(self.responses[idx].clone())
            } else {
                Err(TmuxError::TmuxNotRunning)
            }
        }
    }

    fn ok_output(stdout: &str) -> TmuxOutput {
        TmuxOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            success: true,
        }
    }

    fn fail_output(stderr: &str) -> TmuxOutput {
        TmuxOutput {
            stdout: String::new(),
            stderr: stderr.to_string(),
            success: false,
        }
    }

    #[tokio::test]
    async fn health_check_unavailable_when_tmux_not_running() {
        let executor = MockExecutor::new(vec![fail_output("no server running")]);
        let status = health_check(&executor).await;
        assert!(!status.available);
        assert!(status.version.is_empty());
        assert_eq!(status.session_count, 0);
        assert_eq!(status.client_count, 0);
        assert!(status.uptime_seconds.is_none());
        assert!(status.server_pid.is_none());
    }

    #[tokio::test]
    async fn health_check_returns_full_status() {
        let executor = MockExecutor::new(vec![
            ok_output("tmux 3.4\n"),
            ok_output("dev\nprod\n"),
            ok_output("client1\n"),
            ok_output("12345\t1700000000\n"),
        ]);
        let status = health_check(&executor).await;
        assert!(status.available);
        assert_eq!(status.version, "tmux 3.4");
        assert_eq!(status.session_count, 2);
        assert_eq!(status.client_count, 1);
        assert_eq!(status.server_pid, Some(12345));
        assert!(status.uptime_seconds.is_some());
    }

    #[tokio::test]
    async fn health_check_handles_zero_sessions_and_clients() {
        let executor = MockExecutor::new(vec![
            ok_output("tmux 3.4\n"),
            fail_output("no sessions"),
            ok_output("\n"),
            ok_output("99\t1700000000\n"),
        ]);
        let status = health_check(&executor).await;
        assert!(status.available);
        assert_eq!(status.session_count, 0);
        assert_eq!(status.client_count, 0);
    }

    #[tokio::test]
    async fn health_check_handles_missing_display_message() {
        let executor = MockExecutor::new(vec![
            ok_output("tmux 3.4\n"),
            ok_output("s1\n"),
            ok_output("c1\nc2\n"),
        ]);
        let status = health_check(&executor).await;
        assert!(status.available);
        assert_eq!(status.session_count, 1);
        assert_eq!(status.client_count, 2);
        assert!(status.server_pid.is_none());
        assert!(status.uptime_seconds.is_none());
    }

    #[test]
    fn parse_server_info_valid() {
        let (pid, _uptime) = parse_server_info("12345\t1700000000\n");
        assert_eq!(pid, Some(12345));
        // uptime depends on current time, just check it's Some
    }

    #[test]
    fn parse_server_info_invalid() {
        let (pid, uptime) = parse_server_info("bad\tdata");
        assert!(pid.is_none());
        assert!(uptime.is_none());
    }

    #[test]
    fn parse_server_info_empty() {
        let (pid, uptime) = parse_server_info("");
        assert!(pid.is_none());
        assert!(uptime.is_none());
    }

    #[test]
    fn display_healthy() {
        let status = HealthStatus {
            available: true,
            version: "tmux 3.4".to_string(),
            session_count: 2,
            client_count: 1,
            uptime_seconds: Some(3600),
            server_pid: Some(12345),
        };
        assert_eq!(
            status.to_string(),
            "healthy (tmux 3.4, 2 sessions, 1 clients)"
        );
    }

    #[test]
    fn display_unavailable() {
        let status = HealthStatus::unavailable();
        assert_eq!(status.to_string(), "unavailable");
    }
}
