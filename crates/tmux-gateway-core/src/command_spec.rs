/// A pure specification of a tmux command, decoupled from execution.
///
/// This type is the output of `build_*_command` functions and the input
/// to the executor, enabling command construction to be tested without
/// actually running tmux.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxCommandSpec {
    args: Vec<String>,
}

#[allow(unknown_lints, no_wrapper_functions)]
impl TmuxCommandSpec {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Returns the command arguments as string slices for passing to an executor.
    #[allow(unknown_lints, no_wrapper_functions)]
    pub fn args(&self) -> Vec<&str> {
        self.args.iter().map(|s| s.as_str()).collect()
    }

    /// Returns the tmux sub-command name (first argument), if any.
    #[allow(unknown_lints, no_wrapper_functions)]
    pub fn command_name(&self) -> &str {
        self.args.first().map_or("", |s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_spec_args() {
        let spec = TmuxCommandSpec::new(vec![
            "list-windows".into(),
            "-t".into(),
            "my-session".into(),
        ]);
        assert_eq!(spec.args(), vec!["list-windows", "-t", "my-session"]);
    }

    #[test]
    fn command_spec_command_name() {
        let spec = TmuxCommandSpec::new(vec!["kill-session".into(), "-t".into(), "s".into()]);
        assert_eq!(spec.command_name(), "kill-session");
    }

    #[test]
    fn command_spec_empty() {
        let spec = TmuxCommandSpec::new(vec![]);
        assert_eq!(spec.command_name(), "");
        assert!(spec.args().is_empty());
    }
}
