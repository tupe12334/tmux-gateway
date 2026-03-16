use std::collections::HashMap;

/// Direction for splitting a pane.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Declarative specification for a single pane within a window.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PaneSpec {
    /// Optional command to run in this pane.
    pub command: Option<String>,
    /// How to split from the previous pane to create this one.
    pub split: SplitDirection,
}

/// Declarative specification for a single window within a session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WindowSpec {
    /// Window name.
    pub name: String,
    /// Optional command for the initial pane of this window.
    pub command: Option<String>,
    /// Additional panes to create by splitting.
    #[serde(default)]
    pub panes: Vec<PaneSpec>,
}

/// Declarative specification describing a desired tmux session layout.
///
/// This is a pure value object — serializable, comparable, and testable
/// without a running tmux server. Use [`apply_session_spec`] to realize
/// a spec into actual tmux state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionSpec {
    /// Session name.
    pub name: String,
    /// Windows to create in the session.
    #[serde(default)]
    pub windows: Vec<WindowSpec>,
    /// Environment variables to set on the session.
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_spec_is_clone_debug_partialeq() {
        let spec = SessionSpec {
            name: "dev".to_string(),
            windows: vec![],
            environment: HashMap::new(),
        };
        let cloned = spec.clone();
        assert_eq!(spec, cloned);
        assert_eq!(format!("{spec:?}"), format!("{cloned:?}"));
    }

    #[test]
    fn window_spec_default_panes() {
        let spec = WindowSpec {
            name: "editor".to_string(),
            command: Some("vim".to_string()),
            panes: vec![],
        };
        assert!(spec.panes.is_empty());
    }

    #[test]
    fn pane_spec_equality() {
        let a = PaneSpec {
            command: Some("htop".to_string()),
            split: SplitDirection::Horizontal,
        };
        let b = PaneSpec {
            command: Some("htop".to_string()),
            split: SplitDirection::Horizontal,
        };
        assert_eq!(a, b);

        let c = PaneSpec {
            command: None,
            split: SplitDirection::Vertical,
        };
        assert_ne!(a, c);
    }

    #[test]
    fn split_direction_equality() {
        assert_eq!(SplitDirection::Horizontal, SplitDirection::Horizontal);
        assert_ne!(SplitDirection::Horizontal, SplitDirection::Vertical);
    }

    #[test]
    fn serde_round_trip_json() {
        let spec = SessionSpec {
            name: "my-project".to_string(),
            windows: vec![
                WindowSpec {
                    name: "editor".to_string(),
                    command: Some("vim".to_string()),
                    panes: vec![PaneSpec {
                        command: Some("cargo watch".to_string()),
                        split: SplitDirection::Vertical,
                    }],
                },
                WindowSpec {
                    name: "shell".to_string(),
                    command: None,
                    panes: vec![],
                },
            ],
            environment: HashMap::from([
                ("RUST_LOG".to_string(), "debug".to_string()),
                ("EDITOR".to_string(), "vim".to_string()),
            ]),
        };

        let json = serde_json::to_string_pretty(&spec).unwrap();
        let deserialized: SessionSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, deserialized);
    }

    #[test]
    fn serde_minimal_spec() {
        let json = r#"{"name": "minimal"}"#;
        let spec: SessionSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.name, "minimal");
        assert!(spec.windows.is_empty());
        assert!(spec.environment.is_empty());
    }

    #[test]
    fn full_spec_with_multiple_panes() {
        let spec = SessionSpec {
            name: "monitoring".to_string(),
            windows: vec![WindowSpec {
                name: "dashboard".to_string(),
                command: None,
                panes: vec![
                    PaneSpec {
                        command: Some("htop".to_string()),
                        split: SplitDirection::Horizontal,
                    },
                    PaneSpec {
                        command: Some("tail -f /var/log/syslog".to_string()),
                        split: SplitDirection::Vertical,
                    },
                ],
            }],
            environment: HashMap::new(),
        };
        assert_eq!(spec.windows[0].panes.len(), 2);
    }
}
