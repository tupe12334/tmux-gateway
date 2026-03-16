//! Send keys and capture output example.
//!
//! Demonstrates how to send commands to a tmux pane and read back the
//! output, including capture with options for scrollback history.
//!
//! Run with: `cargo run --example send_and_capture`

use tmux_gateway_core::{
    CaptureOptions, PaneTarget, RealTmuxExecutor, SessionName, capture_pane,
    capture_pane_with_options, kill_session, new_session, send_keys,
};

const SESSION_NAME: &str = "send-capture-example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor::new();
    let session = SessionName::try_from(SESSION_NAME)?;

    // 1. Create a session
    println!("Creating session '{SESSION_NAME}'...");
    let created = new_session(&executor, &session, None).await?;
    println!(
        "Session created: {} (id={}, {} windows)",
        created.name, created.id, created.windows
    );

    let target = PaneTarget::try_from(format!("{SESSION_NAME}:0.0").as_str())?;

    // 2. Send keys (shell commands) to the pane
    println!("\nSending 'echo hello from tmux-gateway' to {target}...");
    send_keys(
        &executor,
        &target,
        &[
            "echo hello from tmux-gateway".to_string(),
            "Enter".to_string(),
        ],
    )
    .await?;

    // Give the shell a moment to execute the command
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 3. Capture pane output (basic)
    let output = capture_pane(&executor, &target).await?;
    println!("\nCaptured pane output:\n{output}");

    // Send another command so there is more history
    println!("Sending 'date' to {target}...");
    send_keys(
        &executor,
        &target,
        &["date".to_string(), "Enter".to_string()],
    )
    .await?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 4. Capture with options (scrollback history, visible region)
    let opts = CaptureOptions {
        start_line: Some(-5),
        end_line: None,
        escape_sequences: false,
    };
    let history_output = capture_pane_with_options(&executor, &target, &opts).await?;
    println!(
        "\nCaptured with options (start_line=-5, including scroll history):\n{history_output}"
    );

    // 5. Cleanup
    println!("\nCleaning up session '{SESSION_NAME}'...");
    kill_session(&executor, &session).await?;
    println!("Done.");

    Ok(())
}
