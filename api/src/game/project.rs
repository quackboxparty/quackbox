//! The trust boundary — `project(&GameState, &GrantSet) -> ClientView`.
//!
//! The room broadcasts full-truth `GameState`; each socket runs THIS function
//! once before sending. It fills each optional `ClientView` section only if the
//! grants permit it. The correct answer is stripped SERVER-SIDE here — never
//! sent-then-hidden in the client. This is the one place security is NOT
//! simplified away.
//!
//! Carries a test: a {Play}-only view must never contain `correct_answer`.
//!
//! TODO: project() + the {Play}-excludes-correct_answer test.

use crate::{
    game::{grants::GrantSet, state::GameState},
    protocol::ClientView,
};

pub fn project(gamestate: &GameState, grants: &GrantSet) -> ClientView {
    let mut players: Vec<String> = gamestate
        .player_slots
        .values()
        .filter(|slot| slot.connected)
        .map(|slot| slot.name.clone())
        .collect();
    players.sort();

    ClientView { players }
}
