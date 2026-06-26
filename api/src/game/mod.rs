//! Game runtime — the domain layer. Knows about channels, timers, and game
//! rules; knows NOTHING about axum or WebSockets. This is the seam that makes
//! the room testable without a socket: drive a room by sending `Command`s down
//! an mpsc and assert on broadcast snapshots in a plain `#[tokio::test]`.
//!
//! RULE: no file in `game/` may `use axum`. If it needs to, the seam is wrong.
//!
//! v1: grid_quiz logic is HARDCODED in `room.rs`. Extract a `Gamemode` trait
//! only when gamemode #2 lands (see docs/architecture.md). One impl ≠ a trait.

pub mod grants;
pub mod judge;
pub mod project;
pub mod room;
pub mod state;
