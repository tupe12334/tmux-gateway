use super::list_panes::TmuxPane;
use super::list_windows::TmuxWindow;
use super::sessions::TmuxSession;
use super::{TmuxError, list_panes, list_sessions, list_windows};
use crate::executor::TmuxExecutor;
use crate::validation::validate_session_target;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowDetail {
    pub window: TmuxWindow,
    pub panes: Vec<TmuxPane>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionDetail {
    pub session: TmuxSession,
    pub windows: Vec<WindowDetail>,
}

#[tracing::instrument(skip(executor))]
pub async fn get_session_detail(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
) -> Result<SessionDetail, TmuxError> {
    validate_session_target(name)?;

    let sessions = list_sessions(executor).await?;
    let session = sessions
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| TmuxError::SessionNotFound(name.to_string()))?;

    let windows = list_windows(executor, name).await?;

    let mut window_details = Vec::with_capacity(windows.len());
    for window in windows {
        let target = format!("{}:{}", name, window.index);
        let panes = list_panes(executor, &target).await?;
        window_details.push(WindowDetail { window, panes });
    }

    Ok(SessionDetail {
        session,
        windows: window_details,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    struct MockDetailExecutor {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        sessions_output: TmuxOutput,
        windows_output: TmuxOutput,
        panes_outputs: Vec<TmuxOutput>,
    }

    impl TmuxExecutor for MockDetailExecutor {
        async fn execute(&self, args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            let mut calls = self.calls.lock().unwrap();
            let call: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            calls.push(call);

            let command = args[0];
            match command {
                "list-sessions" => Ok(self.sessions_output.clone()),
                "list-windows" => Ok(self.windows_output.clone()),
                "list-panes" => {
                    // Return panes based on how many list-panes calls we've made so far
                    let pane_call_index = calls.iter().filter(|c| c[0] == "list-panes").count() - 1;
                    Ok(self
                        .panes_outputs
                        .get(pane_call_index)
                        .cloned()
                        .unwrap_or(TmuxOutput {
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        }))
                }
                _ => Ok(TmuxOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: true,
                }),
            }
        }
    }

    #[tokio::test]
    async fn get_session_detail_composes_hierarchy() {
        let executor = MockDetailExecutor {
            calls: std::sync::Mutex::new(Vec::new()),
            sessions_output: TmuxOutput {
                stdout: "$0\tdev\t2\t1700000000\t0\n".to_string(),
                stderr: String::new(),
                success: true,
            },
            windows_output: TmuxOutput {
                stdout: "@0\t0\tbash\t1\t1\n@1\t1\tvim\t2\t0\n".to_string(),
                stderr: String::new(),
                success: true,
            },
            panes_outputs: vec![
                TmuxOutput {
                    stdout: "%0\t80\t24\t1\t/home\tbash\t1234\n".to_string(),
                    stderr: String::new(),
                    success: true,
                },
                TmuxOutput {
                    stdout: "%1\t80\t12\t1\t/home\tvim\t2345\n%2\t80\t12\t0\t/home\tzsh\t3456\n"
                        .to_string(),
                    stderr: String::new(),
                    success: true,
                },
            ],
        };

        let detail = get_session_detail(&executor, "dev").await.unwrap();

        assert_eq!(detail.session.name, "dev");
        assert_eq!(detail.windows.len(), 2);

        assert_eq!(detail.windows[0].window.name, "bash");
        assert_eq!(detail.windows[0].panes.len(), 1);
        assert_eq!(detail.windows[0].panes[0].id, "%0");

        assert_eq!(detail.windows[1].window.name, "vim");
        assert_eq!(detail.windows[1].panes.len(), 2);
        assert_eq!(detail.windows[1].panes[0].id, "%1");
        assert_eq!(detail.windows[1].panes[1].id, "%2");
    }

    #[tokio::test]
    async fn get_session_detail_session_not_found() {
        let executor = MockDetailExecutor {
            calls: std::sync::Mutex::new(Vec::new()),
            sessions_output: TmuxOutput {
                stdout: "$0\tother\t1\t1700000000\t0\n".to_string(),
                stderr: String::new(),
                success: true,
            },
            windows_output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
            panes_outputs: vec![],
        };

        let result = get_session_detail(&executor, "nonexistent").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TmuxError::SessionNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("expected SessionNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_session_detail_empty_session() {
        let executor = MockDetailExecutor {
            calls: std::sync::Mutex::new(Vec::new()),
            sessions_output: TmuxOutput {
                stdout: "$0\tempty\t0\t1700000000\t0\n".to_string(),
                stderr: String::new(),
                success: true,
            },
            windows_output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
            panes_outputs: vec![],
        };

        let detail = get_session_detail(&executor, "empty").await.unwrap();
        assert_eq!(detail.session.name, "empty");
        assert!(detail.windows.is_empty());
    }

    #[tokio::test]
    async fn get_session_detail_validates_input() {
        let executor = MockDetailExecutor {
            calls: std::sync::Mutex::new(Vec::new()),
            sessions_output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
            windows_output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
            panes_outputs: vec![],
        };

        let result = get_session_detail(&executor, "").await;
        assert!(result.is_err());
    }
}
