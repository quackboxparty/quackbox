//! Wire protocol — the typed contract between frontend and backend.
//!
//! Standalone on purpose: depends on NOTHING in `game` or `http`, so the
//! frontend can import these types (via `ts-rs`) as a pure contract. `game`
//! produces these; `http/ws` serializes them. Dependency flows one way:
//! `protocol` ← `game` ← `http`.
//!
//! - `Command`   — client → server, tagged enum (Join, Buzz, Answer, Next,
//!                 StartGame, Grant, ExtendTimer, …). `#[serde(tag = "kind")]`.
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
use tokio::sync::oneshot;

use crate::game::state::Token;

#[derive(Debug)]
pub enum RoomMessage {
    Join {
        name: String,
        reply: oneshot::Sender<Result<Token, ConnectionError>>,
    },
    Reconnect {
        token: Token,
        reply: oneshot::Sender<Result<Token, ConnectionError>>,
    },
    Client {
        token: Token,
        cmd: Command,
    },
    Disconnect {
        token: Token,
    },
}

#[derive(Debug)]
pub enum ConnectionError {
    NameTaken,
    SlotGone,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Protocol.ts"))]
pub enum ClientMessage {
    Join { name: String },
    Reconnect { token: String },
    Authed { token: String, cmd: Command },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Protocol.ts"))]
pub enum Command {
    Buzz,
    Answer { text: String },
    ExtendTimer { delta_secs: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Protocol.ts"))]
pub enum ServerMessage {
    Joined { token: String },
    Snapshot(ClientView),
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Protocol.ts"))]
pub struct ClientView {
    pub(crate) players: Vec<String>,
    // pub(crate) scoreboard: Option<Scoreboard>,
    // pub(crate) phase: Phase,
    // pub(crate) timer: Option<Deadline>,
    // pub(crate) controls: Option<Controls>,
    // pub(crate) stage: GamemodeView,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ts-rs auto-generates a test from #[ts(export)] that writes Command.ts.
    // This one asserts the serde tagged-enum wire shape the frontend relies on.
    #[test]
    fn answer_uses_internally_tagged_shape() {
        let json = serde_json::to_string(&Command::Answer { text: "42".into() }).unwrap();
        assert_eq!(json, r#"{"kind":"Answer","text":"42"}"#);
    }
}
