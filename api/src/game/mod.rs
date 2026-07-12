//! Game runtime — the domain layer. Knows about channels, timers, and game
//! rules; knows NOTHING about axum or WebSockets. This is the seam that makes
//! the room testable without a socket: drive a room by sending `Command`s down
//! an mpsc and assert on broadcast snapshots in a plain `#[tokio::test]`.
//!
//! RULE: no file in `game/` may `use axum`. If it needs to, the seam is wrong.
//!
//! `GameState.mode: ModeState` carries each mode's play data (`grid_quiz` +
//! `linear` both exist). Behavior (buzz/lockout/timer/scoring) stays hardcoded
//! in the room loop per mode; extract a `Gamemode` trait only when two modes
//! have runtime behavior and the seams are felt (see docs/architecture.md).

pub mod grants;
pub mod judge;
pub mod project;
pub mod room;
pub mod state;
