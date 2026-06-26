//! The WebSocket edge — `GET /ws/{join_code}`.
//!
//! The single endpoint for all live play. This is the ONLY file that touches
//! both a socket and a channel: a thin adapter, bytes ↔ `Command`/`ServerMsg`.
//! No game logic lives here.
//!
//! Flow (see Lesson 10):
//!   1. `WebSocketUpgrade` extractor + `Path(join_code)` + `State`.
//!   2. `ws.on_upgrade(move |socket| handle_socket(...))`.
//!   3. Registry lookup by join_code → `RoomHandle` (cmd_tx clone + state_tx.subscribe()).
//!   4. `socket.split()` → read task (WS → serde → mpsc) + write task
//!      (broadcast → serde → WS); `select!` joins them for teardown.
//!
//! TODO: ws_handler + handle_socket.

use std::sync::Arc;

use axum::{
    Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::any,
};
use futures_util::{SinkExt, StreamExt};

use crate::{AppState, protocol::Command, state::JoinCode};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/ws/{join_code}", any(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(join_code): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, join_code, state))
}

async fn handle_socket(socket: WebSocket, join_code: String, state: Arc<AppState>) {
    let Some(room) = state.rooms.get(&JoinCode(join_code)) else {
        return;
    };

    let command_tx = room.command_tx.clone();
    let mut server_rx = room.server_tx.subscribe();

    let (mut ws_out, mut ws_in) = socket.split();

    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_in.next().await {
            if let Message::Text(txt) = msg
                && let Ok(cmd) = serde_json::from_str::<Command>(&txt)
            {
                let _ = command_tx.send(cmd).await;
            }
        }
    });

    let mut write_task = tokio::spawn(async move {
        while let Ok(snapshot) = server_rx.recv().await {
            let json = serde_json::to_string(&snapshot).unwrap();
            if ws_out.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut read_task => write_task.abort(),
        _ = &mut write_task => read_task.abort()
    }
}
