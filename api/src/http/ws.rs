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
use tokio::sync::oneshot;

use crate::{
    AppState,
    game::{project::project, room::JoinCode, state::Token},
    protocol::{
        ClientMessage, ConnectionError,
        RoomMessage::{self},
        ServerMessage,
    },
};

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
    let disconnect_tx = room.command_tx.clone();
    let mut state_rx = room.state_tx.subscribe();
    let (reply_tx, reply_rx) = oneshot::channel::<Result<Token, ConnectionError>>();

    let (mut ws_out, mut ws_in) = socket.split();

    let mut read_task = tokio::spawn(async move {
        let Some(Ok(Message::Text(txt))) = ws_in.next().await else {
            return;
        };
        let Ok(first) = serde_json::from_str::<ClientMessage>(&txt) else {
            return;
        };
        match first {
            ClientMessage::Join { name } => {
                let _ = command_tx
                    .send(RoomMessage::Join {
                        name,
                        reply: reply_tx,
                    })
                    .await;
            }
            ClientMessage::Reconnect { token } => {
                let _ = command_tx
                    .send(RoomMessage::Reconnect {
                        token: Token(token),
                        reply: reply_tx,
                    })
                    .await;
            }
            ClientMessage::Authed { token, cmd } => {
                let _ = command_tx
                    .send(RoomMessage::Client {
                        token: Token(token),
                        cmd,
                    })
                    .await;
            }
        };
        while let Some(Ok(msg)) = ws_in.next().await {
            if let Message::Text(txt) = msg {
                let Ok(ClientMessage::Authed { token, cmd }) = serde_json::from_str(&txt) else {
                    tracing::debug!("closing socket because of invalid packet");
                    break;
                };
                let _ = command_tx
                    .send(RoomMessage::Client {
                        token: Token(token),
                        cmd,
                    })
                    .await;
            }
        }
    });

    let (token_tx, token_rx) = oneshot::channel::<Token>();

    let mut write_task = tokio::spawn(async move {
        let token = match reply_rx.await {
            Ok(Ok(token)) => token,
            Ok(Err(e)) => {
                let msg = match e {
                    ConnectionError::NameTaken => "Username is already taken!",
                    ConnectionError::SlotGone => "You are not part of the room!",
                };
                let json = serde_json::to_string(&ServerMessage::Error {
                    message: msg.into(),
                })
                .expect("serde infallible");
                let _ = ws_out.send(Message::Text(json.into())).await;
                return;
            }
            Err(_) => return,
        };
        let _ = token_tx.send(token.clone());

        let joined = ServerMessage::Joined {
            token: token.0.clone(),
        };
        let json = serde_json::to_string(&joined).expect("serde infallible");
        if ws_out.send(Message::Text(json.into())).await.is_err() {
            return;
        }

        while let Ok(gamestate) = state_rx.recv().await {
            let grants = gamestate.grants_for(&token);
            let view = match grants {
                Some(grants) => project(&gamestate, grants),
                None => {
                    tracing::debug!("slot gone for live token; closing stream");
                    let json = serde_json::to_string(&ServerMessage::Error {
                        message: "You are no longer in this game.".into(),
                    })
                    .expect("serde infallible");
                    let _ = ws_out.send(Message::Text(json.into())).await;
                    break;
                }
            };
            let json =
                serde_json::to_string(&ServerMessage::Snapshot(view)).expect("serde infallible");
            if ws_out.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut read_task => write_task.abort(),
        _ = &mut write_task => read_task.abort()
    }

    if let Ok(token) = token_rx.await {
        let _ = disconnect_tx.send(RoomMessage::Disconnect { token }).await;
    }
}
