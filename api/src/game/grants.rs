//! Capabilities, not roles. A connection holds a GRANT SET; named roles are
//! grant bundles that compose by union.
//!   Play      — occupy a slot, buzz, answer, own a score.
//!   Present   — see question/options/scoreboard/timer (big screen).
//!   Moderate  — advance game, manage players, SEE CORRECT ANSWERS, controls.
//! "host" = {Moderate, Present}; "playing moderator" = {Moderate, Present, Play}.
//!
//! Grants are stored server-side per reconnect token; the client never asserts
//! its own. Joiner = {Play}; others assigned via a moderator-only Grant command.
//!
//! TODO: Grant enum, GrantSet, helpers.

use std::collections::HashSet;

use serde::Serialize;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize)]
pub enum Grant {
    Play,
    Present,
    Moderate,
}

pub type GrantSet = HashSet<Grant>;
