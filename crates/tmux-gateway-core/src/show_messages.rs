use std::fmt;

use crate::executor::TmuxExecutor;

use super::TmuxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxMessage {
    pub time: String,
    pub message: String,
}

impl fmt::Display for TmuxMessage {
    #[allow(unknown_lints, no_wrapper_functions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.time, self.message)
    }
}

pub(crate) fn parse_message_line(line: &str) -> Result<TmuxMessage, TmuxError> {
    // Format: "HH:MM: message content"
    // The time is separated from the message by ": " after the HH:MM pattern.
    // We split on the first ": " to separate time from message.
    let Some((time, message)) = line.split_once(": ") else {
        return Err(TmuxError::ParseError {
            command: "show-messages".to_string(),
            details: format!("expected 'time: message' format, got: {line}"),
        });
    };
    Ok(TmuxMessage {
        time: time.to_string(),
        message: message.to_string(),
    })
}

/// Retrieve the tmux server message log.
#[tracing::instrument(skip(executor))]
pub async fn show_messages(
    executor: &(impl TmuxExecutor + ?Sized),
) -> Result<Vec<TmuxMessage>, TmuxError> {
    let output = executor.execute(&["show-messages"]).await?;

    if !output.success {
        return Err(TmuxError::from_stderr("show-messages", &output.stderr, ""));
    }

    output
        .stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_message_line)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    // ── Display ──

    #[test]
    fn display_message() {
        let msg = TmuxMessage {
            time: "14:07".to_string(),
            message: "client-123 command: list-sessions".to_string(),
        };
        assert_eq!(msg.to_string(), "14:07: client-123 command: list-sessions");
    }

    // ── Parsing ──

    #[test]
    fn parse_message_line_valid() {
        let msg = parse_message_line("14:07: client-123 command: show-messages").unwrap();
        assert_eq!(msg.time, "14:07");
        assert_eq!(msg.message, "client-123 command: show-messages");
    }

    #[test]
    fn parse_message_line_with_colons_in_message() {
        let msg = parse_message_line("14:06: client-456 message: can't find session: foo").unwrap();
        assert_eq!(msg.time, "14:06");
        assert_eq!(msg.message, "client-456 message: can't find session: foo");
    }

    #[test]
    fn parse_message_line_missing_separator() {
        let result = parse_message_line("no separator here");
        assert!(result.is_err());
    }

    #[test]
    fn parse_messages_multiple_lines() {
        let input = "14:07: msg one\n14:06: msg two\n";
        let messages: Vec<TmuxMessage> = input
            .lines()
            .filter(|l| !l.is_empty())
            .map(parse_message_line)
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].message, "msg one");
        assert_eq!(messages[1].message, "msg two");
    }

    #[test]
    fn parse_messages_empty_input() {
        let messages: Vec<TmuxMessage> = ""
            .lines()
            .filter(|l| !l.is_empty())
            .map(parse_message_line)
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(messages.is_empty());
    }

    // ── Mock executor tests ──

    struct MockExecutor {
        output: TmuxOutput,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            Ok(self.output.clone())
        }
    }

    #[tokio::test]
    async fn show_messages_parses_mock_output() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout:
                    "14:07: client-123 command: show-messages\n14:06: client-456 message: error\n"
                        .to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let messages = show_messages(&executor).await.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].time, "14:07");
        assert_eq!(messages[0].message, "client-123 command: show-messages");
        assert_eq!(messages[1].time, "14:06");
        assert_eq!(messages[1].message, "client-456 message: error");
    }

    #[tokio::test]
    async fn show_messages_empty_output() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let messages = show_messages(&executor).await.unwrap();
        assert!(messages.is_empty());
    }

    #[tokio::test]
    async fn show_messages_error_on_failure() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux".to_string(),
                success: false,
            },
        };
        let result = show_messages(&executor).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
