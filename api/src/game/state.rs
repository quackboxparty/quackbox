//! `GameState` — the full in-memory truth for one room: players, scores
//! (folded from the judgment log, never a mutated counter), current question,
//! timer DEADLINE timestamps (not countdowns), phase. Must be `Clone` (each
//! broadcast subscriber gets a copy).
//!
//! `snapshot()` produces the full-truth value the room broadcasts; per-role
//! stripping happens later in `project`, not here.
//!
//! TODO: GameState, apply(Command), on_timeout, snapshot, score = fold(log).

use std::collections::HashMap;

use uuid::Uuid;

use crate::{game::grants::GrantSet, protocol::Command};

#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub player_slots: HashMap<Token, PlayerSlot>,
}

impl GameState {
    pub fn apply(&mut self, cmd: Command) {
        match cmd {
            _ => {
                todo!()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token(pub String);

impl Token {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSlot {
    pub name: String,
    pub connected: bool,
    pub grants: GrantSet,
}
