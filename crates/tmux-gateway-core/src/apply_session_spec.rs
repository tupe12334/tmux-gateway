use crate::executor::TmuxExecutor;
use crate::session_detail::SessionDetail;
use crate::session_spec::{SessionSpec, SplitDirection};
use crate::validation::{
    PaneTarget, SessionName, WindowTarget, validate_command, validate_env_var_name,
    validate_env_var_value, validate_window_name,
};

use super::TmuxError;

/// Realize a declarative [`SessionSpec`] into actual tmux state.
///
/// Creates the session, windows, panes, and environment variables described
/// by the spec. On partial failure, attempts best-effort rollback by killing
/// the session.
#[tracing::instrument(skip(executor))]
pub async fn apply_session_spec(
    executor: &(impl TmuxExecutor + ?Sized),
    spec: &SessionSpec,
) -> Result<SessionDetail, TmuxError> {
    // Validate all inputs upfront before creating anything
    let session_name = SessionName::try_from(spec.name.as_str())?;
    for window_spec in &spec.windows {
        validate_window_name(&window_spec.name)?;
        if let Some(cmd) = &window_spec.command {
            validate_command(cmd)?;
        }
        for pane_spec in &window_spec.panes {
            if let Some(cmd) = &pane_spec.command {
                validate_command(cmd)?;
            }
        }
    }
    for (name, value) in &spec.environment {
        validate_env_var_name(name)?;
        validate_env_var_value(value)?;
    }

    // Determine the initial command for the session (first window's command, if any)
    let initial_command = spec.windows.first().and_then(|w| w.command.as_deref());

    // Step 1: Create the session
    super::new_session(executor, &session_name, initial_command).await?;

    // Step 2: Set up windows — rename default window to first, create rest
    if let Some((first_window, rest_windows)) = spec.windows.split_first() {
        // Rename the default window to the first window spec's name
        let default_target = WindowTarget::try_from(format!("{}:0", session_name).as_str())?;
        if let Err(e) = super::rename_window(executor, &default_target, &first_window.name).await {
            let _ = super::kill_session(executor, &session_name).await;
            return Err(e);
        }

        // Create panes for the first window
        if let Err(e) = create_panes_for_window(
            executor,
            &session_name,
            &first_window.name,
            &first_window.panes,
        )
        .await
        {
            let _ = super::kill_session(executor, &session_name).await;
            return Err(e);
        }

        // Create remaining windows with their panes
        for window_spec in rest_windows {
            if let Err(e) = super::new_window(
                executor,
                &session_name,
                &window_spec.name,
                window_spec.command.as_deref(),
            )
            .await
            {
                let _ = super::kill_session(executor, &session_name).await;
                return Err(e);
            }

            if let Err(e) = create_panes_for_window(
                executor,
                &session_name,
                &window_spec.name,
                &window_spec.panes,
            )
            .await
            {
                let _ = super::kill_session(executor, &session_name).await;
                return Err(e);
            }
        }
    }

    // Step 3: Set environment variables
    for (name, value) in &spec.environment {
        if let Err(e) = super::set_environment(executor, &session_name, name, value).await {
            let _ = super::kill_session(executor, &session_name).await;
            return Err(e);
        }
    }

    // Step 4: Return the resulting session detail
    super::get_session_detail(executor, &session_name).await
}

/// Create additional panes for a window by splitting the last pane.
async fn create_panes_for_window(
    executor: &(impl TmuxExecutor + ?Sized),
    session: &SessionName,
    window: &str,
    panes: &[crate::session_spec::PaneSpec],
) -> Result<(), TmuxError> {
    // Each new pane splits from the last pane in the window.
    // The first pane (index 0) is created with the window itself.
    for (next_pane_index, pane_spec) in (0_u32..).zip(panes.iter()) {
        let target =
            PaneTarget::try_from(format!("{session}:{window}.{next_pane_index}").as_str())?;
        let horizontal = pane_spec.split == SplitDirection::Horizontal;
        super::split_window(executor, &target, horizontal, pane_spec.command.as_deref()).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;
    use crate::session_spec::{PaneSpec, WindowSpec};
    use std::collections::HashMap;

    /// Mock executor that tracks calls and returns pre-configured outputs
    /// based on the tmux command being executed.
    struct MockSpecExecutor {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        /// Should session creation succeed?
        create_success: bool,
        /// Should window operations succeed?
        window_success: bool,
        /// Should split operations succeed?
        split_success: bool,
        /// Should env operations succeed?
        env_success: bool,
    }

    impl MockSpecExecutor {
        fn new() -> Self {
            Self {
                calls: std::sync::Mutex::new(Vec::new()),
                create_success: true,
                window_success: true,
                split_success: true,
                env_success: true,
            }
        }

        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }

        fn call_commands(&self) -> Vec<String> {
            self.calls
                .lock()
                .unwrap()
                .iter()
                .map(|c| c[0].clone())
                .collect()
        }
    }

    impl TmuxExecutor for MockSpecExecutor {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            let call: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            self.calls.lock().unwrap().push(call);

            let command = args[0];
            match command {
                "has-session" => {
                    // Return true (session exists) only after new-session was called
                    let session_created = self
                        .calls
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|c| c[0] == "new-session");
                    if session_created {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        })
                    } else {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: "can't find session".to_string(),
                            success: false,
                        })
                    }
                }
                "new-session" => {
                    if self.create_success {
                        Ok(TmuxOutput {
                            stdout: "$0\ttest-session\t1\t1700000000\t0\n".to_string(),
                            stderr: String::new(),
                            success: true,
                        })
                    } else {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: "duplicate session: test-session".to_string(),
                            success: false,
                        })
                    }
                }
                "rename-window" => {
                    if self.window_success {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        })
                    } else {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: "window not found".to_string(),
                            success: false,
                        })
                    }
                }
                "new-window" => {
                    if self.window_success {
                        Ok(TmuxOutput {
                            stdout: "@1\t1\ttest\t1\t0\n".to_string(),
                            stderr: String::new(),
                            success: true,
                        })
                    } else {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: "session not found".to_string(),
                            success: false,
                        })
                    }
                }
                "split-window" => {
                    if self.split_success {
                        Ok(TmuxOutput {
                            stdout: "%1\t80\t12\t0\t/home\tbash\t2345\n".to_string(),
                            stderr: String::new(),
                            success: true,
                        })
                    } else {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: "pane too small".to_string(),
                            success: false,
                        })
                    }
                }
                "set-environment" => {
                    if self.env_success {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        })
                    } else {
                        Ok(TmuxOutput {
                            stdout: String::new(),
                            stderr: "session not found".to_string(),
                            success: false,
                        })
                    }
                }
                "kill-session" => Ok(TmuxOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: true,
                }),
                "list-sessions" => Ok(TmuxOutput {
                    stdout: "$0\ttest-session\t2\t1700000000\t0\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                "list-windows" => Ok(TmuxOutput {
                    stdout: "@0\t0\teditor\t1\t1\n@1\t1\tshell\t1\t0\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                "list-panes" => Ok(TmuxOutput {
                    stdout: "%0\t80\t24\t1\t/home\tbash\t1234\n".to_string(),
                    stderr: String::new(),
                    success: true,
                }),
                _ => Ok(TmuxOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: true,
                }),
            }
        }
    }

    #[tokio::test]
    async fn apply_empty_spec_creates_session_only() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_ok());

        let commands = executor.call_commands();
        assert!(commands.contains(&"new-session".to_string()));
        assert!(commands.contains(&"list-sessions".to_string()));
    }

    #[tokio::test]
    async fn apply_spec_with_windows() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![
                WindowSpec {
                    name: "editor".to_string(),
                    command: Some("vim".to_string()),
                    panes: vec![],
                },
                WindowSpec {
                    name: "shell".to_string(),
                    command: None,
                    panes: vec![],
                },
            ],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_ok());

        let commands = executor.call_commands();
        assert!(commands.contains(&"new-session".to_string()));
        assert!(commands.contains(&"rename-window".to_string()));
        assert!(commands.contains(&"new-window".to_string()));
    }

    #[tokio::test]
    async fn apply_spec_with_panes() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![WindowSpec {
                name: "dashboard".to_string(),
                command: None,
                panes: vec![
                    PaneSpec {
                        command: Some("htop".to_string()),
                        split: SplitDirection::Horizontal,
                    },
                    PaneSpec {
                        command: None,
                        split: SplitDirection::Vertical,
                    },
                ],
            }],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_ok());

        let commands = executor.call_commands();
        let split_count = commands.iter().filter(|c| *c == "split-window").count();
        assert_eq!(split_count, 2);
    }

    #[tokio::test]
    async fn apply_spec_with_environment() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![],
            environment: HashMap::from([("RUST_LOG".to_string(), "debug".to_string())]),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_ok());

        let commands = executor.call_commands();
        assert!(commands.contains(&"set-environment".to_string()));
    }

    #[tokio::test]
    async fn apply_spec_validates_session_name() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "".to_string(),
            windows: vec![],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
        // No tmux commands should have been executed
        assert!(executor.calls().is_empty());
    }

    #[tokio::test]
    async fn apply_spec_validates_window_names() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![WindowSpec {
                name: "".to_string(),
                command: None,
                panes: vec![],
            }],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
        assert!(executor.calls().is_empty());
    }

    #[tokio::test]
    async fn apply_spec_validates_commands() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![WindowSpec {
                name: "editor".to_string(),
                command: Some("rm -rf /; bad".to_string()),
                panes: vec![],
            }],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
        assert!(executor.calls().is_empty());
    }

    #[tokio::test]
    async fn apply_spec_validates_env_var_names() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![],
            environment: HashMap::from([("bad name".to_string(), "value".to_string())]),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
        assert!(executor.calls().is_empty());
    }

    #[tokio::test]
    async fn apply_spec_rollback_on_window_failure() {
        let mut executor = MockSpecExecutor::new();
        executor.window_success = false;
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![WindowSpec {
                name: "editor".to_string(),
                command: None,
                panes: vec![],
            }],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_err());

        // Should have called kill-session for rollback
        let commands = executor.call_commands();
        assert!(commands.contains(&"kill-session".to_string()));
    }

    #[tokio::test]
    async fn apply_spec_rollback_on_split_failure() {
        let mut executor = MockSpecExecutor::new();
        executor.split_success = false;
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![WindowSpec {
                name: "dashboard".to_string(),
                command: None,
                panes: vec![PaneSpec {
                    command: None,
                    split: SplitDirection::Horizontal,
                }],
            }],
            environment: HashMap::new(),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_err());

        let commands = executor.call_commands();
        assert!(commands.contains(&"kill-session".to_string()));
    }

    #[tokio::test]
    async fn apply_spec_rollback_on_env_failure() {
        let mut executor = MockSpecExecutor::new();
        executor.env_success = false;
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![],
            environment: HashMap::from([("RUST_LOG".to_string(), "debug".to_string())]),
        };

        let result = apply_session_spec(&executor, &spec).await;
        assert!(result.is_err());

        let commands = executor.call_commands();
        assert!(commands.contains(&"kill-session".to_string()));
    }

    #[tokio::test]
    async fn apply_spec_first_window_command_passed_to_session() {
        let executor = MockSpecExecutor::new();
        let spec = SessionSpec {
            name: "test-session".to_string(),
            windows: vec![WindowSpec {
                name: "editor".to_string(),
                command: Some("vim".to_string()),
                panes: vec![],
            }],
            environment: HashMap::new(),
        };

        let _ = apply_session_spec(&executor, &spec).await;

        let calls = executor.calls();
        let new_session_call = calls.iter().find(|c| c[0] == "new-session").unwrap();
        // The command "vim" should be part of the new-session args
        assert!(new_session_call.contains(&"vim".to_string()));
    }
}
