use axum::Router;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::routing::get;
use serde::Deserialize;
use std::time::Duration;

use crate::tmux::{self, RealTmuxExecutor};

#[derive(Deserialize)]
struct WsParams {
    #[serde(default = "default_interval_ms")]
    interval_ms: u64,
}

fn default_interval_ms() -> u64 {
    500
}

async fn ws_pane_handler(
    Path(target): Path<String>,
    Query(params): Query<WsParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let interval = Duration::from_millis(params.interval_ms.max(100));
    ws.on_upgrade(move |socket| handle_pane_stream(socket, target, interval))
}

async fn handle_pane_stream(mut socket: WebSocket, target: String, interval: Duration) {
    let mut last_content = String::new();
    let mut ticker = tokio::time::interval(interval);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                match tmux::capture_pane(&RealTmuxExecutor, &target).await {
                    Ok(content) => {
                        if content != last_content {
                            last_content = content.clone();
                            if socket.send(Message::Text(content.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = socket
                            .send(Message::Close(Some(CloseFrame {
                                code: 1011,
                                reason: e.to_string().into(),
                            })))
                            .await;
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

pub fn router() -> Router {
    Router::new().route("/ws/pane/{target}", get(ws_pane_handler))
}
