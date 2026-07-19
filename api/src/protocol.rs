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

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::game::{
    grants::{Grant, GrantSet},
    judge::Verdict,
    state::{GridQuizPhase, Token},
};

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
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum ClientMessage {
    Join { name: String },
    Reconnect { token: String },
    Authed { token: String, cmd: Command },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum Command {
    // ── lifecycle ──
    StartGame,
    PickCell { category: usize, point: usize },
    CloseQuestion,
    Next,
    EndGame,
    // ── answering ──
    Buzz,
    Answer { text: String },
    Rule { player: String, verdict: Verdict },
    // ── controls ──
    Grant { player: String, grants: GrantSet },
    ExtendTimer { delta_secs: u32 },
    Kick { player: String },
}

impl Command {
    pub fn required_grant(&self) -> Option<Grant> {
        match self {
            Command::Kick { .. }
            | Command::StartGame
            | Command::EndGame
            | Command::Grant { .. }
            | Command::Next
            | Command::Rule { .. } => Some(Grant::Moderate),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum ServerMessage {
    Joined { token: String },
    Snapshot(ClientView),
    Error { message: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct ClientView {
    pub(crate) players: BTreeMap<String, PlayerView>,
    // pub(crate) phase: Phase,
    // pub(crate) timer: Option<Deadline>,
    // pub(crate) controls: Option<Controls>,
    pub(crate) stage: GamemodeView,
    pub(crate) question: Option<QuestionView>,
    pub(crate) judgment_log: Vec<JudgmentView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct PlayerView {
    pub(crate) grants: GrantSet,
    pub(crate) score: i32,
    pub(crate) connected: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct JudgmentView {
    pub game_idx: usize,
    pub player: String,
    pub question_id: String,
    /// `None` for spoken answers (moderator verdict stands alone).
    pub submission: Option<String>,
    pub verdict: Verdict,
    /// Resolved award (cell value, half on steal, penalty…). The fold sums
    /// this — steal/half math can't be re-derived from a single entry, so the
    /// outcome is recorded once at append (steal logic lives in one place).
    pub points: i32,
    /// Index of the log entry this supersedes, if revising a prior ruling.
    pub supersedes: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum GamemodeView {
    GridQuiz(GridQuizView),
    Linear {},
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct GridQuizView {
    pub phase: GridQuizPhase,
    pub categories: Vec<String>,
    pub points: Vec<u32>,
    pub used: Vec<Vec<bool>>,
    pub current_category: Option<String>,
    pub current_points: Option<u32>,
    pub active_picker: Option<String>,
    pub floored: Option<String>,
    pub locked_out: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct QuestionView {
    pub prompt: PromptView,
    pub variant: VariantView,
    pub answer: Option<AnswerView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct PromptView {
    pub text: String,
    pub media: Option<MediaView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum MediaSrc {
    /// `local:` (server-resolved to a URL) or `url:` direct remote — load into
    /// the kind's media element.
    Url(String),
    /// `youtube:` id — render an iframe embed, not a media element.
    Youtube(String),
}

/// Per-kind: each variant exposes only the fields valid for that kind — Audio
/// carries no dimensions, Image carries no playback timing. Type error, not a
/// stray `Option`, if they're mismatched.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum MediaView {
    Image {
        src: MediaSrc,
        alt: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Video {
        src: MediaSrc,
        alt: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
        duration_ms: Option<u32>,
        start_ms: Option<u32>,
        end_ms: Option<u32>,
    },
    Audio {
        src: MediaSrc,
        alt: Option<String>,
        duration_ms: Option<u32>,
        start_ms: Option<u32>,
        end_ms: Option<u32>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum VariantView {
    MultipleChoice { choices: Vec<ChoiceView> },
    Open,
    TrueFalse,
    NumericInput,
    Range { min: f64, max: f64, step: f64 },
    Order { items: Vec<OrderItemView> },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct ChoiceView {
    pub id: String,
    pub text: String,
    pub media: Option<MediaView>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct OrderItemView {
    pub id: String,
    pub text: String,
    pub media: Option<MediaView>,
}

/// The reveal block — `Some` if `Moderate || phase == Reveal`. Wraps the
/// per-variant correctness plus the shared explanation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct AnswerView {
    pub correctness: CorrectnessView,
    pub explanation: Option<String>,
}

// TODO: this needs to be translated
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub enum CorrectnessView {
    MultipleChoice { correct_ids: Vec<String> },
    Open { accepted: Vec<String> },
    TrueFalse { correct: bool },
    Numeric { value: f64, tolerance: f64 },
    Order { positions: Vec<OrderPositionView> },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export, export_to = "Protocol.ts"))]
pub struct OrderPositionView {
    pub id: String,
    pub position: u32,
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
