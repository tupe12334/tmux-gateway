/// Log level for domain-level structured logging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// A port for structured operation-level logging in the domain layer.
///
/// This trait follows the ports & adapters pattern: the domain defines the
/// logging interface, and the imperative shell provides the concrete
/// implementation (bridging to `tracing`, `log`, or any framework).
///
/// Object-safe by design — can be used as `&dyn LogPort`.
pub trait LogPort: Send + Sync {
    /// Log a message for a domain operation.
    fn log(&self, level: LogLevel, operation: &str, message: &str);

    /// Log a message for a domain operation targeting a specific resource.
    fn log_with_target(&self, level: LogLevel, operation: &str, target: &str, message: &str);
}

/// No-op implementation for testing and when logging is disabled.
///
/// Zero overhead — all methods are empty.
pub struct NoopLog;

impl LogPort for NoopLog {
    fn log(&self, _: LogLevel, _: &str, _: &str) {}
    fn log_with_target(&self, _: LogLevel, _: &str, _: &str, _: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_log_does_not_panic() {
        let log = NoopLog;
        log.log(LogLevel::Debug, "test-op", "debug message");
        log.log(LogLevel::Info, "test-op", "info message");
        log.log(LogLevel::Warn, "test-op", "warn message");
        log.log(LogLevel::Error, "test-op", "error message");
        log.log_with_target(LogLevel::Info, "test-op", "sess:0", "targeted message");
        log.log_with_target(LogLevel::Debug, "test-op", "sess:0.1", "pane message");
    }

    #[test]
    fn log_port_is_object_safe() {
        let log: &dyn LogPort = &NoopLog;
        log.log(LogLevel::Info, "test-op", "object-safe call");
        log.log_with_target(
            LogLevel::Warn,
            "test-op",
            "target",
            "object-safe targeted call",
        );
    }

    #[test]
    fn log_level_debug_traits() {
        // Verify Clone, Copy, Debug, PartialEq, Eq
        let level = LogLevel::Info;
        let cloned = level;
        assert_eq!(level, cloned);
        assert_ne!(LogLevel::Debug, LogLevel::Error);
        assert_eq!(format!("{:?}", LogLevel::Warn), "Warn");
    }

    #[test]
    fn log_port_can_be_sent_across_threads() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoopLog>();
    }

    /// A recording implementation used to verify domain operations log correctly.
    struct RecordingLog {
        entries: std::sync::Mutex<Vec<(LogLevel, String, Option<String>, String)>>,
    }

    impl RecordingLog {
        fn new() -> Self {
            Self {
                entries: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn entries(&self) -> Vec<(LogLevel, String, Option<String>, String)> {
            self.entries.lock().unwrap().clone()
        }
    }

    impl LogPort for RecordingLog {
        fn log(&self, level: LogLevel, operation: &str, message: &str) {
            self.entries.lock().unwrap().push((
                level,
                operation.to_string(),
                None,
                message.to_string(),
            ));
        }

        fn log_with_target(&self, level: LogLevel, operation: &str, target: &str, message: &str) {
            self.entries.lock().unwrap().push((
                level,
                operation.to_string(),
                Some(target.to_string()),
                message.to_string(),
            ));
        }
    }

    #[test]
    fn recording_log_captures_entries() {
        let log = RecordingLog::new();
        log.log(LogLevel::Info, "new-session", "creating session 'foo'");
        log.log_with_target(LogLevel::Warn, "kill-window", "sess:0", "window killed");

        let entries = log.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, LogLevel::Info);
        assert_eq!(entries[0].1, "new-session");
        assert!(entries[0].2.is_none());
        assert_eq!(entries[0].3, "creating session 'foo'");
        assert_eq!(entries[1].0, LogLevel::Warn);
        assert_eq!(entries[1].2, Some("sess:0".to_string()));
    }
}
