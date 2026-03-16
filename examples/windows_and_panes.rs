//! Window and pane management example.
//!
//! Demonstrates managing windows and panes using `tmux-gateway-core`:
//! creating a session, adding windows, splitting panes, listing panes,
//! selecting a layout, resizing panes, and cleaning up.
//!
//! Run with: `cargo run --example windows_and_panes`

use tmux_gateway_core::{
    PaneLayout, PaneTarget, RealTmuxExecutor, ResizeDirection, SessionName, WindowTarget,
    kill_session, list_panes, list_windows, new_session, new_window, resize_pane, select_layout,
    split_window,
};

const SESSION_NAME: &str = "windows-panes-example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor::new();
    let session = SessionName::try_from(SESSION_NAME)?;

    // 1. Create a session
    println!("Creating session '{SESSION_NAME}'...");
    let created = new_session(&executor, &session, None, None).await?;
    println!(
        "Session created: {} (id={}, {} windows)",
        created.name, created.id, created.windows
    );

    // 2. Create new windows
    println!("\nCreating additional windows...");
    let editor_win = new_window(&executor, &session, "editor", None).await?;
    println!(
        "  Created window: {} (index={})",
        editor_win.name, editor_win.index
    );

    let server_win = new_window(&executor, &session, "server", None).await?;
    println!(
        "  Created window: {} (index={})",
        server_win.name, server_win.index
    );

    // 3. Split windows into panes
    println!("\nSplitting 'editor' window...");
    let editor_target =
        PaneTarget::try_from(format!("{SESSION_NAME}:{}.0", editor_win.index).as_str())?;
    let h_pane = split_window(&executor, &editor_target, true, None).await?;
    println!("  Horizontal split -> pane {}", h_pane.id);

    let v_pane = split_window(&executor, &editor_target, false, None).await?;
    println!("  Vertical split -> pane {}", v_pane.id);

    // 4. List panes
    println!("\nPanes in each window:");
    let windows = list_windows(&executor, &session).await?;
    for window in &windows {
        let target = WindowTarget::try_from(format!("{SESSION_NAME}:{}", window.index).as_str())?;
        let panes = list_panes(&executor, &target).await?;
        println!(
            "  {} (index={}) — {} panes:",
            window.name,
            window.index,
            panes.len()
        );
        for pane in &panes {
            println!("    {pane}");
        }
    }

    // 5. Select layout
    let editor_win_target =
        WindowTarget::try_from(format!("{SESSION_NAME}:{}", editor_win.index).as_str())?;
    println!("\nApplying tiled layout to 'editor' window...");
    select_layout(&executor, &editor_win_target, PaneLayout::Tiled).await?;
    println!("  Layout set to tiled.");

    // 6. Resize panes
    let resize_target =
        PaneTarget::try_from(format!("{SESSION_NAME}:{}.0", editor_win.index).as_str())?;
    println!("\nResizing pane {resize_target}...");
    resize_pane(&executor, &resize_target, ResizeDirection::Right(5)).await?;
    println!("  Resized right by 5 cells.");
    resize_pane(&executor, &resize_target, ResizeDirection::Down(3)).await?;
    println!("  Resized down by 3 cells.");

    // 7. Cleanup
    println!("\nCleaning up session '{SESSION_NAME}'...");
    kill_session(&executor, &session).await?;
    println!("Done.");

    Ok(())
}
