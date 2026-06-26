//! Shared application state, held in an `Arc` and handed to handlers via the
//! axum `State` extractor.
//!
//! Sits ABOVE both `http` and `game`: both need it, neither owns it.
//! - `data`   — read-only loaded content (`Arc` is enough; no mutation).
//! - `rooms`  — the live-game registry: `DashMap<JoinCode, RoomHandle>`. The
//!              ONLY mutable shared state; DashMap gives concurrent interior
//!              mutability so no lock is written by hand.
//! - `config` — server config (host/port, creation secret).
//!
//! TODO: move `AppState` here out of `main.rs`; add the `rooms` registry.

use dashmap::DashMap;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

use crate::{
    config::AppConfig,
    data::LoadedDataset,
    protocol::{Command, ServerMessage},
};

pub struct AppState {
    pub config: AppConfig,
    pub data: LoadedDataset,
    pub rooms: DashMap<JoinCode, RoomHandle>,
}

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
    pub command_tx: mpsc::Sender<Command>,
    pub server_tx: broadcast::Sender<ServerMessage>,
}
