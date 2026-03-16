//! Event-driven session example.
//!
//! Demonstrates creating a tmux session with event broadcasting,
//! subscribing to events, and reacting to session/window changes.
//!
//! Run with: `cargo run --example events`

use tmux_gateway_core::{
    EventSender, RealTmuxExecutor, TmuxEvent, kill_session, new_session_with_events, new_window,
    send_keys,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let executor = RealTmuxExecutor::new();
    let session_name = "events-demo";

    // Create a broadcast channel for tmux events.
    let (tx, mut rx): (EventSender, _) = tokio::sync::broadcast::channel(16);

    // A second subscriber to show multi-subscriber support.
    let mut rx2 = tx.subscribe();

    // Spawn a task that listens for events and reacts to them.
    let listener = tokio::spawn(async move {
        println!("[listener] Waiting for events...");
        while let Ok(event) = rx.recv().await {
            match &event {
                TmuxEvent::SessionCreated { name } => {
                    println!("[listener] Session created: {name}");
                }
                TmuxEvent::WindowCreated { session, name } => {
                    println!("[listener] Window '{name}' created in session '{session}'");
                }
                TmuxEvent::KeysSent { target } => {
                    println!("[listener] Keys sent to {target}");
                }
                other => {
                    println!("[listener] Event: {other:?}");
                }
            }
        }
    });

    // Spawn a second listener to demonstrate fan-out.
    let logger = tokio::spawn(async move {
        while let Ok(event) = rx2.recv().await {
            println!("[logger]   {event:?}");
        }
    });

    // --- Create a session with event broadcasting ---
    println!("Creating session '{session_name}'...");
    let session = new_session_with_events(&executor, session_name, None, Some(&tx)).await?;
    println!(
        "Session ready: id={}, windows={}",
        session.id, session.windows
    );

    // --- Create a window and broadcast the event manually ---
    println!("\nCreating a second window...");
    let window = new_window(&executor, session_name, "worker", None).await?;
    let _ = tx.send(TmuxEvent::WindowCreated {
        session: session_name.to_string(),
        name: window.name.clone(),
    });

    // --- Send keys and broadcast the event ---
    let target = format!("{session_name}:{}.0", window.index);
    let keys = ["echo hello from events demo".to_string()];
    send_keys(&executor, &target, &keys).await?;
    let _ = tx.send(TmuxEvent::KeysSent {
        target: target.clone(),
    });

    // Allow listeners to process all events.
    tokio::task::yield_now().await;

    // --- Cleanup ---
    println!("\nCleaning up...");
    kill_session(&executor, session_name).await?;
    let _ = tx.send(TmuxEvent::SessionKilled {
        name: session_name.to_string(),
    });

    // Drop the sender so listeners terminate.
    drop(tx);
    let _ = listener.await;
    let _ = logger.await;

    println!("Done.");
    Ok(())
}
