//! `GameState` — the full in-memory truth for one room: players, scores
//! (folded from the judgment log, never a mutated counter), current question,
//! timer DEADLINE timestamps (not countdowns), phase. Must be `Clone` (each
//! broadcast subscriber gets a copy).
//!
//! `snapshot()` produces the full-truth value the room broadcasts; per-role
//! stripping happens later in `project`, not here.
//!
//! TODO: GameState, apply(Command), on_timeout, snapshot, score = fold(log).
