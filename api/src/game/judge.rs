//! Adjudication — the judge axis, orthogonal to gamemode. `(Gamemode × Judge)`.
//!
//! `Judge::verdict(submission, &question_data) -> Verdict`, where
//! `Verdict ∈ {Correct, Incorrect, Void, Pending}`. Both v1 impls feed the same
//! append-only judgment log, so revision/override is identical regardless of
//! origin (`score = fold(log)`).
//!   Auto      — static matcher (correct: true, numeric answer + tolerance).
//!   Moderator — submission lands Pending; a Moderate-granted human rules.
//! Selection per question: f(answer_input, question_kind). Spoken → Moderator,
//! typed+auto-matchable → Auto, else Moderator.
//!
//! Deferred (same trait): Quorum, Llm.
//!
//! TODO: Judge trait, Auto, Moderator (Verdict defined below).

use serde::{Deserialize, Serialize};

/// Adjudication outcome. `Pending` is transient (awaits a moderator ruling or a
/// timer-driven auto-verdict); the rest are final. Stored in the judgment log;
/// `score = fold(log)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Verdict.ts"))]
pub enum Verdict {
    Correct,
    Incorrect,
    Void,
    Pending,
}
