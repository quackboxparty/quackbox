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
    let data = Arc::clone(&state.data);

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

        let mut joined_seen = false;
        while let Ok(gamestate) = state_rx.recv().await {
            let grants = gamestate.player_slots.grants_for(&token);
            let view = match grants {
                Some(grants) => {
                    joined_seen = true;
                    project(&data, &gamestate, grants)
                }
                // snapshot predates our join - skip, own join broadcast is still queued
                None if !joined_seen => continue,
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dashmap::DashMap;
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

    use super::*;
    use crate::{config::AppConfig, game::room::spawn_room, protocol::Command, state::AppState};

    type Client = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

    /// Real server on port 0 with one room; returns the ws URL for that room.
    async fn start_server() -> String {
        let data = Arc::new(crate::data::load("../data").expect("load ../data"));
        let game = data
            .games
            .values()
            .next()
            .expect("example dataset has a game")
            .item
            .clone();
        let code = JoinCode("TEST42".into());
        let handle = spawn_room(code.clone(), game, Arc::clone(&data));
        let state = Arc::new(AppState {
            config: AppConfig::default(),
            data,
            rooms: DashMap::new(),
        });
        state.rooms.insert(code, handle);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router().with_state(state)).await.unwrap();
        });
        format!("ws://{addr}/ws/TEST42")
    }

    async fn connect(url: &str) -> Client {
        let (ws, _) = connect_async(url).await.expect("ws connect");
        ws
    }

    async fn send_msg(ws: &mut Client, msg: &ClientMessage) {
        let json = serde_json::to_string(msg).unwrap();
        ws.send(tungstenite::Message::Text(json.into()))
            .await
            .unwrap();
    }

    async fn recv_msg(ws: &mut Client) -> ServerMessage {
        loop {
            let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
                .await
                .expect("timed out waiting for server message")
                .expect("socket closed")
                .expect("ws error");
            if let tungstenite::Message::Text(txt) = msg {
                return serde_json::from_str(&txt).expect("valid ServerMessage");
            }
        }
    }

    /// Regression: a connection subscribes to the room broadcast at connect
    /// time, but joins later (user typing at the name dialog). Snapshots
    /// broadcast in between predate its slot and must be skipped — they used
    /// to be mistaken for a kick, killing every join into an active room.
    #[tokio::test]
    async fn join_survives_snapshots_broadcast_while_at_name_dialog() {
        let url = start_server().await;

        let mut a = connect(&url).await;
        // let the server run handle_socket for A so its broadcast rx exists
        tokio::time::sleep(Duration::from_millis(50)).await;

        // B joins while A sits at the name dialog → a snapshot without A
        // queues up in A's rx
        let mut b = connect(&url).await;
        send_msg(&mut b, &ClientMessage::Join { name: "host".into() }).await;
        assert!(matches!(recv_msg(&mut b).await, ServerMessage::Joined { .. }));
        assert!(matches!(recv_msg(&mut b).await, ServerMessage::Snapshot(_)));

        send_msg(&mut a, &ClientMessage::Join { name: "karl".into() }).await;
        assert!(matches!(recv_msg(&mut a).await, ServerMessage::Joined { .. }));
        match recv_msg(&mut a).await {
            ServerMessage::Snapshot(view) => assert!(view.players.contains_key("karl")),
            other => panic!("expected snapshot containing karl, got {other:?}"),
        }
    }

    /// The stale-snapshot skip must not swallow real kicks: once a player has
    /// seen itself in a snapshot, a snapshot without it means kicked → Error.
    #[tokio::test]
    async fn kicked_player_gets_error() {
        let url = start_server().await;

        // first joiner gets the Moderate grant
        let mut host = connect(&url).await;
        send_msg(&mut host, &ClientMessage::Join { name: "host".into() }).await;
        let ServerMessage::Joined { token } = recv_msg(&mut host).await else {
            panic!("expected Joined");
        };

        let mut karl = connect(&url).await;
        send_msg(&mut karl, &ClientMessage::Join { name: "karl".into() }).await;
        assert!(matches!(recv_msg(&mut karl).await, ServerMessage::Joined { .. }));

        send_msg(
            &mut host,
            &ClientMessage::Authed {
                token,
                cmd: Command::Kick {
                    player: "karl".into(),
                },
            },
        )
        .await;

        loop {
            match recv_msg(&mut karl).await {
                ServerMessage::Snapshot(_) => continue,
                ServerMessage::Error { message } => {
                    assert_eq!(message, "You are no longer in this game.");
                    break;
                }
                other => panic!("expected kick error, got {other:?}"),
            }
        }
    }
}
