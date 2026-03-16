//! Demonstrates how to get, set, and list tmux options at every scope
//! (global, session, window).
//!
//! Run with:
//!
//! ```sh
//! cargo run --example options
//! ```
//!
//! Prerequisites: a running tmux server with at least one session.

use tmux_gateway_core::{OptionScope, RealTmuxExecutor, get_option, list_options, set_option};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor::new();

    // ── 1. Global option ────────────────────────────────────────────
    println!("=== Global options ===");

    let opt = get_option(&executor, "base-index", OptionScope::Global, None).await?;
    println!("base-index (global): {}", opt.value);

    let globals = list_options(&executor, OptionScope::Global, None).await?;
    println!("Total global options: {}\n", globals.len());

    // ── 2. Session option ───────────────────────────────────────────
    println!("=== Session options ===");

    // Read the current global value of "status", set it at session scope,
    // verify, then restore by setting it back.
    let global_status = get_option(&executor, "status", OptionScope::Global, None).await?;
    println!("status (global default): {}", global_status.value);

    let new_value = if global_status.value == "on" {
        "off"
    } else {
        "on"
    };
    set_option(&executor, "status", new_value, OptionScope::Session, None).await?;
    let changed = get_option(&executor, "status", OptionScope::Session, None).await?;
    println!(
        "status (session, after set to {new_value}): {}",
        changed.value
    );

    // Restore to original value
    set_option(
        &executor,
        "status",
        &global_status.value,
        OptionScope::Session,
        None,
    )
    .await?;
    println!("status (session, restored to {})", global_status.value);

    let session_opts = list_options(&executor, OptionScope::Session, None).await?;
    println!("Total session options: {}\n", session_opts.len());

    // ── 3. Window option ────────────────────────────────────────────
    println!("=== Window options ===");

    let win_opt = get_option(&executor, "mode-keys", OptionScope::Window, None).await?;
    println!("mode-keys (window): {}", win_opt.value);

    let window_opts = list_options(&executor, OptionScope::Window, None).await?;
    println!("Total window options: {}", window_opts.len());

    println!("\nDone!");
    Ok(())
}
