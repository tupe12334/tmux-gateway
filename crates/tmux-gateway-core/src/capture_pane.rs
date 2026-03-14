use crate::executor::TmuxExecutor;

use super::TmuxError;

/// Options for controlling what content is captured from a pane.
#[derive(Debug, Clone, Default)]
pub struct CaptureOptions {
    /// Starting line number (-S flag). Negative values reach into scroll history.
    pub start_line: Option<i32>,
    /// Ending line number (-E flag).
    pub end_line: Option<i32>,
    /// Include escape sequences in output (-e flag).
    pub escape_sequences: bool,
}

pub async fn capture_pane(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
) -> Result<String, TmuxError> {
    capture_pane_with_options(executor, target, &CaptureOptions::default()).await
}

pub async fn capture_pane_with_options(
    executor: &(impl TmuxExecutor + ?Sized),
    target: &str,
    opts: &CaptureOptions,
) -> Result<String, TmuxError> {
    let mut args: Vec<&str> = vec!["capture-pane", "-p", "-t", target];

    let start_str;
    if let Some(start) = opts.start_line {
        start_str = start.to_string();
        args.push("-S");
        args.push(&start_str);
    }

    let end_str;
    if let Some(end) = opts.end_line {
        end_str = end.to_string();
        args.push("-E");
        args.push(&end_str);
    }

    if opts.escape_sequences {
        args.push("-e");
    }

    let output = executor.execute(&args).await?;
    if !output.success {
        return Err(TmuxError::from_stderr(
            "capture-pane",
            &output.stderr,
            target,
        ));
    }
    Ok(output.stdout)
}
