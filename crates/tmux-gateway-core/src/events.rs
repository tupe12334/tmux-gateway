/// Domain events emitted by tmux operations.
///
/// These are pure domain concepts with no transport or serialization concerns.
/// API layers can subscribe and forward events via their respective transport
/// (WebSocket, gRPC streaming, GraphQL subscriptions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmuxEvent {
    SessionCreated { name: String },
    SessionKilled { name: String },
    WindowCreated { session: String, name: String },
    WindowKilled { target: String },
    PaneCreated { target: String },
    PaneKilled { target: String },
    KeysSent { target: String },
    SessionRenamed { old_name: String, new_name: String },
    WindowRenamed { target: String, new_name: String },
}

/// Broadcast sender for domain events. Supports multiple subscribers.
pub type EventSender = tokio::sync::broadcast::Sender<TmuxEvent>;

/// Broadcast receiver for domain events.
pub type EventReceiver = tokio::sync::broadcast::Receiver<TmuxEvent>;

/// Create a new event channel with the given capacity.
pub fn event_channel(capacity: usize) -> (EventSender, EventReceiver) {
    tokio::sync::broadcast::channel(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_channel_send_receive() {
        let (tx, mut rx) = event_channel(16);
        tx.send(TmuxEvent::SessionCreated {
            name: "test".to_string(),
        })
        .unwrap();

        let event = rx.try_recv().unwrap();
        assert_eq!(
            event,
            TmuxEvent::SessionCreated {
                name: "test".to_string()
            }
        );
    }

    #[test]
    fn event_channel_multiple_subscribers() {
        let (tx, mut rx1) = event_channel(16);
        let mut rx2 = tx.subscribe();

        tx.send(TmuxEvent::SessionKilled {
            name: "s1".to_string(),
        })
        .unwrap();

        assert_eq!(
            rx1.try_recv().unwrap(),
            TmuxEvent::SessionKilled {
                name: "s1".to_string()
            }
        );
        assert_eq!(
            rx2.try_recv().unwrap(),
            TmuxEvent::SessionKilled {
                name: "s1".to_string()
            }
        );
    }

    #[test]
    fn event_clone_and_debug() {
        let event = TmuxEvent::WindowCreated {
            session: "main".to_string(),
            name: "editor".to_string(),
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
        assert!(!format!("{event:?}").is_empty());
    }

    #[test]
    fn no_receivers_send_returns_err() {
        let (tx, rx) = event_channel(16);
        drop(rx);
        let result = tx.send(TmuxEvent::KeysSent {
            target: "s:w.0".to_string(),
        });
        assert!(result.is_err());
    }
}
