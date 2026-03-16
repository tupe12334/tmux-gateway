//! Basic session workflow example.
//!
//! Demonstrates the core session lifecycle using `tmux-gateway-core`:
//! creating a session, listing sessions, capturing pane output, and
//! killing the session.
//!
//! Run with: `cargo run --example basic_session`

use tmux_gateway_core::{RealTmuxExecutor, capture_pane, kill_session, list_sessions, new_session};

const SESSION_NAME: &str = "basic-session-example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor::new();

    // 1. Create a new session
    println!("Creating session '{SESSION_NAME}'...");
    let session = new_session(&executor, SESSION_NAME, None).await?;
    println!(
        "Session created: {} (id={}, {} windows)",
        session.name, session.id, session.windows
    );

    // 2. List all sessions
    let sessions = list_sessions(&executor).await?;
    println!("\nAll sessions:");
    for s in &sessions {
        println!("  {s}");
    }

    // 3. Capture pane output from the default pane
    let target = format!("{SESSION_NAME}:0.0");
    let output = capture_pane(&executor, &target).await?;
    println!("\nCaptured pane output ({target}):");
    println!("{output}");

    // 4. Kill the session
    println!("Cleaning up session '{SESSION_NAME}'...");
    kill_session(&executor, SESSION_NAME).await?;
    println!("Done.");

    Ok(())
}
