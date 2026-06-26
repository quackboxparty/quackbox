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
//! TODO: Judge trait, Verdict, Auto, Moderator.
