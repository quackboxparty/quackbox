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

use crate::{
    config::AppConfig,
    data::Dataset,
    game::room::{JoinCode, RoomHandle},
};

pub struct AppState {
    pub config: AppConfig,
    pub data: Dataset,
    pub rooms: DashMap<JoinCode, RoomHandle>,
}
