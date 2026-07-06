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

use crate::{
    data::GameConfig,
    game::grants::{Grant, GrantSet},
    protocol::Command,
};

#[derive(Clone, Debug)]
pub struct GameState {
    pub game: GameConfig,
    pub player_slots: HashMap<Token, PlayerSlot>,
}

impl GameState {
    pub fn apply(&mut self, token: Token, cmd: Command) {
        match cmd {
            Command::Kick { player } => {
                if self
                    .grants_for(&token)
                    .is_none_or(|grants| !grants.contains(&Grant::Moderate))
                {
                    tracing::info!(
                        "Token '{}' tried to kick without right permissions",
                        token.0
                    );
                    return;
                }

                self.player_slots.retain(|_, v| v.name != player);
            }
            _ => {
                todo!()
            }
        }
    }

    pub(crate) fn grants_for(&self, token: &Token) -> Option<&GrantSet> {
        match self.player_slots.get(token) {
            Some(slot) => Some(&slot.grants),
            None => None,
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
