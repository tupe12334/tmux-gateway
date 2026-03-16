//! Batch session setup example.
//!
//! Demonstrates creating a tmux session with multiple pre-configured windows
//! in a single call using `create_session_with_windows`, then lists the
//! resulting windows and panes before cleaning up.
//!
//! Run with: `cargo run --example batch_setup`

use tmux_gateway_core::{
    RealTmuxExecutor, create_session_with_windows, kill_session, list_panes, list_windows,
};

const SESSION_NAME: &str = "batch-setup-example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor;

    let window_names: Vec<String> = vec!["editor".into(), "server".into(), "logs".into()];

    // Create a session with multiple windows in one call
    println!("Creating session '{SESSION_NAME}' with windows: {window_names:?}");
    let session = create_session_with_windows(&executor, SESSION_NAME, &window_names).await?;
    println!(
        "Session created: {} (id={}, {} windows)",
        session.name, session.id, session.windows
    );

    // List windows
    let windows = list_windows(&executor, SESSION_NAME).await?;
    println!("\nWindows:");
    for window in &windows {
        println!("  {window}");

        // List panes in each window
        let target = format!("{SESSION_NAME}:{}", window.index);
        let panes = list_panes(&executor, &target).await?;
        for pane in &panes {
            println!("    {pane}");
        }
    }

    // Cleanup
    println!("\nCleaning up session '{SESSION_NAME}'...");
    kill_session(&executor, SESSION_NAME).await?;
    println!("Done.");

    Ok(())
}
