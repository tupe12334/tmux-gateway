//! Basic session workflow example.
//!
//! Demonstrates the core session lifecycle using `tmux-gateway-core`:
//! creating a session, listing sessions, capturing pane output, and
//! killing the session.
//!
//! Run with: `cargo run --example basic_session`

use tmux_gateway_core::{
    PaneTarget, RealTmuxExecutor, SessionName, capture_pane, kill_session, list_sessions,
    new_session,
};

const SESSION_NAME: &str = "basic-session-example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor::new();
    let session = SessionName::try_from(SESSION_NAME)?;

    // 1. Create a new session
    println!("Creating session '{SESSION_NAME}'...");
    let created = new_session(&executor, &session, None).await?;
    println!(
        "Session created: {} (id={}, {} windows)",
        created.name, created.id, created.windows
    );

    // 2. List all sessions
    let sessions = list_sessions(&executor).await?;
    println!("\nAll sessions:");
    for s in &sessions {
        println!("  {s}");
    }

    // 3. Capture pane output from the default pane
    let target = PaneTarget::try_from(format!("{SESSION_NAME}:0.0").as_str())?;
    let output = capture_pane(&executor, &target).await?;
    println!("\nCaptured pane output ({target}):");
    println!("{output}");

    // 4. Kill the session
    println!("Cleaning up session '{SESSION_NAME}'...");
    kill_session(&executor, &session).await?;
    println!("Done.");

    Ok(())
}
