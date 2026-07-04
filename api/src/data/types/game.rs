//! Game config types — parsed from `data/games/*.yaml`.
//!
//! Lives in data/types (not game/) to avoid circular deps: loader parses YAML
//! into these, game layer consumes the resolved config.

use std::collections::{HashMap, HashSet};

use garde::Validate;
use serde::{Deserialize, Serialize};

use crate::data::{PackFilter, valid_game_id};

/// Top-level game config, parsed from `data/games/*.yaml`.
#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    #[garde(custom(valid_game_id))]
    pub id: String, // game_<slug>
    pub title: String,       // translatable
    pub description: String, // translatable
    #[serde(default)]
    pub auto_advance: bool, // auto-start next game in chain
    #[garde(length(min = 1), custom(valid_game_entries))]
    pub games: Vec<GameEntry>, // ordered sequence
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameOverlay {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub games: Vec<GameEntryOverlay>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameEntryOverlay {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub board: Option<BoardOverlay>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardOverlay {
    #[serde(default)]
    pub categories: Vec<BoardCategoryOverlay>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardCategoryOverlay {
    #[serde(default)]
    pub name: Option<String>,
}

/// A single game in the chain. Each has its own mode, rules, and content.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameEntry {
    /// Grid-based, Jeopardy-style quiz with NxM board.
    GridQuiz(GridQuizGame),
    /// Linear sequence of questions (Kahoot-style).
    Linear(LinearGame),
}

// ── Game-specific payloads ──────────────────────────────────────────────────

/// Grid quiz: inline board definition.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridQuizGame {
    pub title: String, // translatable
    pub rules: Rules,
    pub board: Board,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    #[serde(default)]
    pub difficulty_map: Option<HashMap<u32, Vec<String>>>,
    pub points: Vec<u32>,
    pub categories: Vec<BoardCategory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardCategory {
    pub name: String,
    #[serde(default)]
    pub question_ids: Option<HashMap<u32, String>>,
    #[serde(default)]
    pub pack_ref: Option<String>,
    #[serde(default)]
    pub filter: Option<PackFilter>,
}

/// Linear quiz: resolved question list.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinearGame {
    pub title: String, // translatable
    pub rules: Rules,
    pub questions: LinearSource,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum LinearSource {
    /// Explicit list of question IDs.
    Questions { question_ids: Vec<String> },
    /// Resolve from a pack.
    Pack { pack_id: String },
    /// Dynamic filter.
    Filter { filter: PackFilter },
}

/// Per-entry game rules. Complete, no merging with defaults.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub buzz_policy: BuzzPolicy,
    pub scoring_mode: ScoringMode,
    pub lockout_policy: LockoutPolicy,
    pub steal_policy: StealPolicy,
    pub judge: Judge,
    #[serde(default = "default_question_timer")]
    pub question_timer_secs: u32,
    #[serde(default = "default_answer_timer")]
    pub answer_timer_secs: u32,
}

fn default_question_timer() -> u32 {
    30
}
fn default_answer_timer() -> u32 {
    15
}

/// Who gets the floor when a question goes live.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BuzzPolicy {
    OpenFloor,
    TurnOrder,
    Broadcast,
}

/// How points are decided for a question.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ScoringMode {
    FirstCorrect,
    AllGrade,
    ClosestWins,
}

/// What happens after a wrong answer.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LockoutPolicy {
    None,
    ThisQuestion,
    ThisRound,
}

/// After wrong answer, does floor reopen?
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum StealPolicy {
    #[serde(rename = "none")]
    None,
    OpenFloor,
    RoundLimited,
}

/// How answers are judged.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Judge {
    Auto,
    Moderator,
}

impl GameEntry {
    /// Validate rules for this entry. Returns error if invalid combo found.
    // ponytail: inline validation, not a trait. One struct, skip abstraction.
    pub fn validate(&self) -> Result<(), String> {
        let rules = match self {
            GameEntry::GridQuiz(g) => &g.rules,
            GameEntry::Linear(g) => &g.rules,
        };

        validate_rules(rules)?;

        match self {
            GameEntry::GridQuiz(g) => validate_grid_quiz(g),
            GameEntry::Linear(g) => validate_linear(g),
        }
    }
}

fn valid_game_entries(entries: &[GameEntry], _ctx: &()) -> garde::Result {
    for (idx, entry) in entries.iter().enumerate() {
        if let Err(msg) = entry.validate() {
            return Err(garde::Error::new(format!("game[{idx}]: {msg}")));
        }
    }
    Ok(())
}

fn validate_rules(rules: &Rules) -> Result<(), String> {
    // Broadcast + FirstCorrect = nonsense (everyone answers, no "first")
    if rules.buzz_policy == BuzzPolicy::Broadcast && rules.scoring_mode == ScoringMode::FirstCorrect
    {
        return Err("buzz_policy:broadcast incompatible with scoring_mode:first_correct".into());
    }
    // Broadcast + Steal != None = nonsense (everyone already answered, can't steal)
    if rules.buzz_policy == BuzzPolicy::Broadcast && rules.steal_policy != StealPolicy::None {
        return Err("buzz_policy:broadcast requires steal_policy:none".into());
    }
    // Lockout on broadcast = pointless (everyone answers, no one is locked from answering)
    if rules.buzz_policy == BuzzPolicy::Broadcast && rules.lockout_policy != LockoutPolicy::None {
        return Err("buzz_policy:broadcast requires lockout_policy:none".into());
    }

    if rules.question_timer_secs == 0 {
        return Err("question_timer_secs must be greater than 0".into());
    }
    if rules.answer_timer_secs == 0 {
        return Err("answer_timer_secs must be greater than 0".into());
    }

    Ok(())
}

fn validate_grid_quiz(g: &GridQuizGame) -> Result<(), String> {
    if g.board.points.len() < 2 {
        return Err("grid_quiz board must define at least 2 point values".into());
    }
    if g.board.categories.len() < 2 {
        return Err("grid_quiz board must define at least 2 categories".into());
    }
    if g.board.points.windows(2).any(|w| w[0] >= w[1]) {
        return Err("grid_quiz board points must be strictly increasing and unique".into());
    }

    let point_set: HashSet<u32> = g.board.points.iter().copied().collect();
    if let Some(diff_map) = &g.board.difficulty_map {
        for point in diff_map.keys() {
            if !point_set.contains(point) {
                return Err(format!(
                    "grid_quiz difficulty_map key {point} must be present in board.points"
                ));
            }
        }
    }

    let mut explicit_qids = HashSet::new();
    for (idx, cat) in g.board.categories.iter().enumerate() {
        if cat.question_ids.is_none() && cat.pack_ref.is_none() && cat.filter.is_none() {
            return Err(format!(
                "grid_quiz category[{idx}] must define at least one of: question_ids, pack_ref, filter"
            ));
        }

        if let Some(question_ids) = &cat.question_ids {
            for point in question_ids.keys() {
                if !point_set.contains(point) {
                    return Err(format!(
                        "grid_quiz category[{idx}] question_ids key {point} must be present in board.points"
                    ));
                }
            }
            for qid in question_ids.values() {
                if !explicit_qids.insert(qid.as_str()) {
                    return Err(format!(
                        "grid_quiz explicit question id '{qid}' is duplicated across board"
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_linear(g: &LinearGame) -> Result<(), String> {
    if let LinearSource::Questions { question_ids } = &g.questions {
        if question_ids.is_empty() {
            return Err(
                "linear questions.source=questions requires at least one question_id".into(),
            );
        }

        let mut seen = HashSet::new();
        for qid in question_ids {
            if !seen.insert(qid.as_str()) {
                return Err(format!(
                    "linear questions.source=questions has duplicate question id '{qid}'"
                ));
            }
        }
    }

    Ok(())
}
