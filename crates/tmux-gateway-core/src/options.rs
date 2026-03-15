use crate::executor::TmuxExecutor;
use crate::validation::{validate_option_name, validate_option_scope_target};

use super::TmuxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxOption {
    pub name: String,
    pub value: String,
    pub scope: OptionScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionScope {
    Global,
    Session,
    Window,
}

impl OptionScope {
    pub fn as_flag(&self) -> &'static str {
        match self {
            Self::Global => "-g",
            Self::Session => "-s",
            Self::Window => "-w",
        }
    }
}

impl std::fmt::Display for OptionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "global"),
            Self::Session => write!(f, "session"),
            Self::Window => write!(f, "window"),
        }
    }
}

#[tracing::instrument(skip(executor))]
pub async fn get_option(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
    scope: OptionScope,
    target: Option<&str>,
) -> Result<TmuxOption, TmuxError> {
    validate_option_name(name)?;
    if let Some(t) = target {
        validate_option_scope_target(scope, t)?;
    }
    let mut args = vec!["show-options", scope.as_flag()];
    if let Some(t) = target {
        args.push("-t");
        args.push(t);
    }
    args.push(name);

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "show-options",
            &output.stderr,
            target.unwrap_or(""),
        ));
    }

    let line = output.stdout.trim();
    let value = if let Some((_opt_name, val)) = line.split_once(' ') {
        val.to_string()
    } else {
        String::new()
    };

    Ok(TmuxOption {
        name: name.to_string(),
        value,
        scope,
    })
}

#[tracing::instrument(skip(executor))]
pub async fn set_option(
    executor: &(impl TmuxExecutor + ?Sized),
    name: &str,
    value: &str,
    scope: OptionScope,
    target: Option<&str>,
) -> Result<(), TmuxError> {
    validate_option_name(name)?;
    if let Some(t) = target {
        validate_option_scope_target(scope, t)?;
    }
    let mut args = vec!["set-option", scope.as_flag()];
    if let Some(t) = target {
        args.push("-t");
        args.push(t);
    }
    args.push(name);
    args.push(value);

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "set-option",
            &output.stderr,
            target.unwrap_or(""),
        ));
    }
    Ok(())
}

#[tracing::instrument(skip(executor))]
pub async fn list_options(
    executor: &(impl TmuxExecutor + ?Sized),
    scope: OptionScope,
    target: Option<&str>,
) -> Result<Vec<TmuxOption>, TmuxError> {
    if let Some(t) = target {
        validate_option_scope_target(scope, t)?;
    }
    let mut args = vec!["show-options", scope.as_flag()];
    if let Some(t) = target {
        args.push("-t");
        args.push(t);
    }

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "show-options",
            &output.stderr,
            target.unwrap_or(""),
        ));
    }

    let options = output
        .stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, value) = if let Some((n, v)) = line.split_once(' ') {
                (n.to_string(), v.to_string())
            } else {
                (line.to_string(), String::new())
            };
            TmuxOption {
                name,
                value,
                scope,
            }
        })
        .collect();

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    struct MockExecutor {
        output: TmuxOutput,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            Ok(self.output.clone())
        }
    }

    // ── OptionScope ──

    #[test]
    fn scope_as_flag() {
        assert_eq!(OptionScope::Global.as_flag(), "-g");
        assert_eq!(OptionScope::Session.as_flag(), "-s");
        assert_eq!(OptionScope::Window.as_flag(), "-w");
    }

    #[test]
    fn scope_display() {
        assert_eq!(OptionScope::Global.to_string(), "global");
        assert_eq!(OptionScope::Session.to_string(), "session");
        assert_eq!(OptionScope::Window.to_string(), "window");
    }

    // ── get_option ──

    #[tokio::test]
    async fn get_option_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "status on\n".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let opt = get_option(&executor, "status", OptionScope::Global, None)
            .await
            .unwrap();
        assert_eq!(opt.name, "status");
        assert_eq!(opt.value, "on");
        assert_eq!(opt.scope, OptionScope::Global);
    }

    #[tokio::test]
    async fn get_option_empty_value() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "prefix\n".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let opt = get_option(&executor, "prefix", OptionScope::Session, None)
            .await
            .unwrap();
        assert_eq!(opt.value, "");
    }

    #[tokio::test]
    async fn get_option_invalid_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = get_option(&executor, "", OptionScope::Global, None).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn get_option_command_failure() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            },
        };
        let result = get_option(&executor, "status", OptionScope::Global, None).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }

    // ── set_option ──

    #[tokio::test]
    async fn set_option_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = set_option(&executor, "status", "off", OptionScope::Global, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn set_option_invalid_name() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let result = set_option(&executor, "bad;name", "val", OptionScope::Global, None).await;
        assert!(matches!(result, Err(TmuxError::Validation(_))));
    }

    #[tokio::test]
    async fn set_option_command_failure() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: nosession".to_string(),
                success: false,
            },
        };
        let result = set_option(
            &executor,
            "status",
            "on",
            OptionScope::Session,
            Some("nosession"),
        )
        .await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    // ── list_options ──

    #[tokio::test]
    async fn list_options_success() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: "status on\nbase-index 0\nprefix C-b\n".to_string(),
                stderr: String::new(),
                success: true,
            },
        };
        let opts = list_options(&executor, OptionScope::Global, None)
            .await
            .unwrap();
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0].name, "status");
        assert_eq!(opts[0].value, "on");
        assert_eq!(opts[1].name, "base-index");
        assert_eq!(opts[1].value, "0");
        assert_eq!(opts[2].name, "prefix");
        assert_eq!(opts[2].value, "C-b");
    }

    #[tokio::test]
    async fn list_options_empty() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            },
        };
        let opts = list_options(&executor, OptionScope::Window, None)
            .await
            .unwrap();
        assert!(opts.is_empty());
    }

    #[tokio::test]
    async fn list_options_command_failure() {
        let executor = MockExecutor {
            output: TmuxOutput {
                stdout: String::new(),
                stderr: "no server running on /tmp/tmux-1000/default".to_string(),
                success: false,
            },
        };
        let result = list_options(&executor, OptionScope::Global, None).await;
        assert!(matches!(result, Err(TmuxError::TmuxNotRunning)));
    }
}
