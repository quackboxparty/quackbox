//! Wire protocol — the typed contract between frontend and backend.
//!
//! Standalone on purpose: depends on NOTHING in `game` or `http`, so the
//! frontend can import these types (via `ts-rs`) as a pure contract. `game`
//! produces these; `http/ws` serializes them. Dependency flows one way:
//! `protocol` ← `game` ← `http`.
//!
//! - `Command`   — client → server, tagged enum (Join, Buzz, Answer, Next,
//!                 StartGame, Grant, ExtendTimer, …). `#[serde(tag = "type")]`.
//! - `ServerMsg` — server → client envelope (Snapshot(ClientView), Joined,
//!                 Error, …).
//! - `ClientView`— the per-role projection result: one type with OPTIONAL
//!                 sections (question?, buzzer?, answer_input?, correct_answer?,
//!                 controls?, scoreboard?). `project` fills only what grants allow.
//!
//! ts-rs is a DEV-dependency, so the `TS` derive only exists under `cargo test`.
//! Gate it with `#[cfg_attr(test, ...)]`: bindings are a test-time artifact,
//! ts-rs never ships in the release binary. `cargo test` regenerates the `.ts`
//! files into `src/lib/bindings` (set via api/.cargo/config.toml).
//!
//! TODO: define ServerMsg, ClientView; flesh out Command's full variant set.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Command.ts"))]
pub enum Command {
    Join { name: String },
    Buzz,
    Answer { text: String },
    ExtendTimer { delta_secs: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "ServerMessage.ts"))]
pub enum ServerMessage {
    Snapshot(ClientView),
    Notify(Notification),
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "ClientView.ts"))]
pub struct ClientView {
    pub(crate) players: Vec<String>,
    // pub(crate) scoreboard: Option<Scoreboard>,
    // pub(crate) phase: Phase,
    // pub(crate) timer: Option<Deadline>,
    // pub(crate) controls: Option<Controls>,
    // pub(crate) stage: GamemodeView,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Notification.ts"))]
pub enum Notification {
    Joined,
    Left,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ts-rs auto-generates a test from #[ts(export)] that writes Command.ts.
    // This one asserts the serde tagged-enum wire shape the frontend relies on.
    #[test]
    fn answer_uses_internally_tagged_shape() {
        let json = serde_json::to_string(&Command::Answer { text: "42".into() }).unwrap();
        assert_eq!(json, r#"{"type":"Answer","text":"42"}"#);
    }
}
