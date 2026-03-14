use serde::{Deserialize, Serialize};

use crate::executor::TmuxExecutor;
use crate::validation::{validate_option_name, validate_option_scope_target};

use super::TmuxError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TmuxOption {
    pub name: String,
    pub value: String,
    pub scope: OptionScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

pub async fn get_option(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    name: &str,
    scope: OptionScope,
) -> Result<String, TmuxError> {
    validate_option_name(name)?;
    validate_option_scope_target(target, scope)?;

    let mut args: Vec<&str> = vec!["show-option", "-v", scope.as_flag()];
    if scope != OptionScope::Global {
        args.push("-t");
        args.push(target);
    }
    args.push(name);

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr("show-option", &output.stderr, target));
    }
    Ok(output.stdout.trim().to_string())
}

pub async fn set_option(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    name: &str,
    value: &str,
    scope: OptionScope,
) -> Result<(), TmuxError> {
    validate_option_name(name)?;
    validate_option_scope_target(target, scope)?;

    let mut args: Vec<&str> = vec!["set-option", scope.as_flag()];
    if scope != OptionScope::Global {
        args.push("-t");
        args.push(target);
    }
    args.push(name);
    args.push(value);

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr("set-option", &output.stderr, target));
    }
    Ok(())
}

pub async fn list_options(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    scope: OptionScope,
) -> Result<Vec<TmuxOption>, TmuxError> {
    validate_option_scope_target(target, scope)?;

    let mut args: Vec<&str> = vec!["show-options", scope.as_flag()];
    if scope != OptionScope::Global {
        args.push("-t");
        args.push(target);
    }

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "show-options",
            &output.stderr,
            target,
        ));
    }

    let options = output
        .stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let (name, value) = line.split_once(' ')?;
            Some(TmuxOption {
                name: name.to_string(),
                value: value.to_string(),
                scope,
            })
        })
        .collect();

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::TmuxOutput;

    struct MockExecutor {
        result: Result<TmuxOutput, TmuxError>,
    }

    impl TmuxExecutor for MockExecutor {
        async fn execute(&self, _args: &[&str]) -> Result<TmuxOutput, TmuxError> {
            match &self.result {
                Ok(output) => Ok(output.clone()),
                Err(e) => Err(TmuxError::CommandFailed {
                    command: "mock".to_string(),
                    stderr: e.to_string(),
                }),
            }
        }
    }

    #[tokio::test]
    async fn get_option_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "on\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = get_option(&executor, "", "mouse", OptionScope::Global).await;
        assert_eq!(result.unwrap(), "on");
    }

    #[tokio::test]
    async fn set_option_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_option(&executor, "", "mouse", "on", OptionScope::Global).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_options_success() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "mouse on\nhistory-limit 2000\nstatus-left [#S]\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = list_options(&executor, "", OptionScope::Global).await;
        let opts = result.unwrap();
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0].name, "mouse");
        assert_eq!(opts[0].value, "on");
        assert_eq!(opts[1].name, "history-limit");
        assert_eq!(opts[1].value, "2000");
        assert_eq!(opts[2].name, "status-left");
        assert_eq!(opts[2].value, "[#S]");
    }

    #[tokio::test]
    async fn get_option_session_scope() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: "2000\n".to_string(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = get_option(&executor, "my-session", "history-limit", OptionScope::Session).await;
        assert_eq!(result.unwrap(), "2000");
    }

    #[tokio::test]
    async fn get_option_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = get_option(&executor, "", "$(inject)", OptionScope::Global).await;
        assert!(matches!(result, Err(TmuxError::InvalidTarget(_))));
    }

    #[tokio::test]
    async fn set_option_invalid_name() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = set_option(&executor, "", "foo;bar", "on", OptionScope::Global).await;
        assert!(matches!(result, Err(TmuxError::InvalidTarget(_))));
    }

    #[tokio::test]
    async fn get_option_session_not_found() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: "session not found: nosession".to_string(),
                success: false,
            }),
        };
        let result = get_option(&executor, "nosession", "mouse", OptionScope::Session).await;
        assert!(matches!(result, Err(TmuxError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn list_options_empty_output() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = list_options(&executor, "", OptionScope::Global).await;
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn session_scope_requires_target() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = get_option(&executor, "", "mouse", OptionScope::Session).await;
        assert!(matches!(result, Err(TmuxError::InvalidTarget(_))));
    }

    #[tokio::test]
    async fn window_scope_requires_target() {
        let executor = MockExecutor {
            result: Ok(TmuxOutput {
                stdout: String::new(),
                stderr: String::new(),
                success: true,
            }),
        };
        let result = get_option(&executor, "", "mouse", OptionScope::Window).await;
        assert!(matches!(result, Err(TmuxError::InvalidTarget(_))));
    }
}
