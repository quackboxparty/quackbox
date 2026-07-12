//! The room actor — one owning tokio task per live game.
//!
//! `spawn_room()` creates the channels and spawns the task; returns a
//! `RoomHandle { cmd_tx, state_tx }` for the registry. The task loop is a
//! `tokio::select!` racing: next `Command` (mpsc), the timer deadline
//! (`sleep_until`), and shutdown — mutate state, broadcast a snapshot, loop.
//! Sole owner of `GameState`, so mutation needs no lock; first-buzz-wins falls
//! out of mpsc ordering. On exit, removes its own entry from the registry.
//!
//! v1: grid_quiz buzz/lockout/timer/scoring policy lives directly in this loop.
//!
//! TODO: RoomHandle, spawn_room, the select! loop.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use rand::RngExt;
use tokio::sync::{broadcast, mpsc};

use crate::{
    data::{Dataset, GameConfig, GameMode, build_board},
    game::{
        grants::Grant::{self},
        state::{Cell, GameState, GridQuizState, LinearState, ModeState, PlayerSlot, Token},
    },
    protocol::{ConnectionError, RoomMessage},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinCode(pub String);

const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789"; // no 0 O 1 I L
const LEN: usize = 6;

impl JoinCode {
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        JoinCode(
            (0..LEN)
                .map(|_| {
                    let i = rng.random_range(0..ALPHABET.len());
                    ALPHABET[i] as char
                })
                .collect(),
        )
    }
}

#[derive(Clone)]
pub struct RoomHandle {
    pub command_tx: mpsc::Sender<RoomMessage>,
    pub state_tx: broadcast::Sender<Arc<GameState>>,
}

pub fn spawn_room(code: JoinCode, game_config: GameConfig, data: Arc<Dataset>) -> RoomHandle {
    let (room_msg_tx, mut room_msg_rx) = mpsc::channel::<RoomMessage>(64);
    let (state_tx, _) = broadcast::channel::<Arc<GameState>>(16);

    let state_tx_loop = state_tx.clone();
    tokio::spawn(async move {
        let mode = match &game_config
            .games
            .first()
            .expect("a room without a game")
            .mode
        {
            GameMode::GridQuiz(game) => {
                let seed = rand::rng().random::<u64>();
                let cells = build_board(&data, &game.board, seed)
                    .into_iter()
                    .map(|row| row.into_iter().map(Cell::from).collect())
                    .collect();
                ModeState::GridQuiz(GridQuizState::build(cells))
            }
            GameMode::Linear(_) => ModeState::Linear(LinearState::default()),
        };

        let mut state = GameState {
            game_config,
            current_game_idx: 0,
            player_slots: HashMap::new(),
            mode,
            judgment_log: Vec::new(),
        };

        while let Some(room_msg) = room_msg_rx.recv().await {
            tracing::info!("room msg {:?}", &room_msg);
            match room_msg {
                RoomMessage::Join { name, reply } => {
                    let taken = state.player_slots.values().any(|slot| slot.name == name);
                    if taken {
                        let _ = reply.send(Err(ConnectionError::NameTaken));
                        continue;
                    }

                    let mut grants = HashSet::from([Grant::Play]);
                    if state.player_slots.is_empty() {
                        grants.insert(Grant::Moderate);
                    }

                    let token = Token::generate();
                    state.player_slots.insert(
                        token.clone(),
                        PlayerSlot {
                            name,
                            connected: true,
                            grants,
                        },
                    );

                    let _ = reply.send(Ok(token));
                }
                RoomMessage::Reconnect { token, reply } => {
                    let Some(slot) = state.player_slots.get_mut(&token) else {
                        let _ = reply.send(Err(ConnectionError::SlotGone));
                        continue;
                    };

                    slot.connected = true;

                    let _ = reply.send(Ok(token));
                }
                RoomMessage::Disconnect { token } => {
                    if let Some(slot) = state.player_slots.get_mut(&token) {
                        slot.connected = false;
                    }
                }
                RoomMessage::Client { token, cmd } => {
                    tracing::info!("command {:?} received in room {}", cmd, &code.0);

                    state.apply(token, cmd);
                }
            }

            let _ = state_tx_loop.send(Arc::new(state.clone()));
        }
    });

    RoomHandle {
        command_tx: room_msg_tx,
        state_tx,
    }
}
