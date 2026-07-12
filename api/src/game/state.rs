//! `GameState` — the full in-memory truth for one room: players, scores
//! (folded from the judgment log, never a mutated counter), current question,
//! timer DEADLINE timestamps (not countdowns), phase. Must be `Clone` (each
//! broadcast subscriber gets a copy).
//!
//! `snapshot()` produces the full-truth value the room broadcasts; per-role
//! stripping happens later in `project`, not here.
//!
//! TODO: GameState, apply(Command), on_timeout, snapshot, score = fold(log).

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    data::{
        BoardGrid, Game, GameConfig,
        GameMode::{self, Linear},
    },
    game::{
        grants::{Grant, GrantSet},
        judge::Verdict,
    },
    protocol::Command,
};

#[derive(Clone, Debug)]
pub struct GameState {
    pub game_config: GameConfig,
    /// Which entry in the game chain (`game.games[..]`) is currently live.
    pub current_game_idx: usize,
    pub player_slots: HashMap<Token, PlayerSlot>,
    pub mode: ModeState,
    /// Global, append-only, spans all rounds. `score = fold(judgment_log)`.
    pub judgment_log: Vec<Judgment>,
}

impl GameState {
    pub fn apply(&mut self, token: Token, cmd: Command) {
        match cmd {
            Command::Kick { player } => {
                if self
                    .grants_for(&token)
                    .is_none_or(|grants| !grants.contains(&Grant::Moderate))
                {
                    tracing::info!(
                        "Token '{}' tried to kick without right permissions",
                        token.0
                    );
                    return;
                }

                self.player_slots.retain(|_, v| v.name != player);
            }
            other => self.mode.apply(&self.player_slots, token, other),
        }
    }

    pub(crate) fn grants_for(&self, token: &Token) -> Option<&GrantSet> {
        match self.player_slots.get(token) {
            Some(slot) => Some(&slot.grants),
            None => None,
        }
    }

    pub(crate) fn player_name(&self, token: &Token) -> Option<String> {
        match self.player_slots.get(token) {
            Some(slot) => Some(slot.name.clone()),
            None => None,
        }
    }

    pub(crate) fn current_game(&self) -> &Game {
        self.game_config
            .games
            .get(self.current_game_idx)
            .expect("current game idx does not yiled a game")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token(pub String);

impl Token {
    pub fn generate() -> Self {
        Self(Uuid::new_v4().into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerSlot {
    pub name: String,
    pub connected: bool,
    pub grants: GrantSet,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModeState {
    GridQuiz(GridQuizState),
    Linear(LinearState),
}

impl ModeState {
    pub fn apply(&mut self, player_slots: &HashMap<Token, PlayerSlot>, token: Token, cmd: Command) {
        match self {
            ModeState::GridQuiz(modestate) => match cmd {
                Command::StartGame => {
                    // TODO: maybe shuffle this
                    let rotation: VecDeque<Token> = player_slots
                        .iter()
                        .filter(|(_, player)| player.connected)
                        .map(|(token, _)| token.clone())
                        .collect();
                    modestate.picker_rotation = rotation;

                    modestate.active_picker = modestate.picker_rotation.front().cloned();

                    modestate.phase = GridQuizPhase::BoardSelect;
                }
                Command::PickCell { category, point } => {
                    modestate.current = modestate
                        .cells
                        .get(category)
                        .and_then(|column| column.get(point))
                        .and_then(|cell| match cell {
                            Cell::Open(question) => Some(CurrentCell {
                                category,
                                point,
                                question_id: question.clone(),
                            }),
                            Cell::Used(_) => {
                                tracing::warn!(category, point, "pick on used cell");
                                None
                            }
                            Cell::Empty => {
                                tracing::warn!(category, point, "pick on empty cell");
                                None
                            }
                        });

                    // TODO: this should respect different flooring strategies like OpenBuzz or
                    // TurnBased etc.
                    modestate.floored_player = modestate.active_picker.clone();
                    modestate.active_picker = None;
                    if let Some(token) = modestate.picker_rotation.pop_front() {
                        modestate.picker_rotation.push_back(token);
                    }
                }
                _ => todo!("other gridquiz cmds not implemented yet"),
            },
            ModeState::Linear(_) => todo!("Linear not implemented yet"),
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridQuizState {
    pub phase: GridQuizPhase,
    /// Whose turn to choose a cell. Meaningful only while `phase == BoardSelect`.
    pub active_picker: Option<Token>,
    /// Who may answer right now. `None` = buzz open; `Some` = that player has
    /// the floor. How it relates to `active_picker` depends on the answer
    /// policy (turn-order: floored == picker on pick; open-floor: first buzz).
    // TODO: this will probably something every mode has, so maybe refactor this
    pub floored_player: Option<Token>,
    /// Answered wrong this question — barred from re-buzzing until it resets.
    pub locked_out: HashSet<Token>,
    /// Cell + question currently in play; `None` while on the board.
    pub current: Option<CurrentCell>,
    /// Picking turn order. Shuffled at `StartGame`; advanced per picker policy.
    pub picker_rotation: VecDeque<Token>,
    pub cells: Vec<Vec<Cell>>,
}

impl GridQuizState {
    pub fn build(cells: Vec<Vec<Cell>>) -> Self {
        Self {
            phase: GridQuizPhase::Lobby,
            active_picker: None,
            floored_player: None,
            locked_out: HashSet::new(),
            current: None,
            picker_rotation: VecDeque::new(),
            cells,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cell {
    Open(String),
    Used(String),
    Empty,
}

impl From<Option<String>> for Cell {
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(id) => Cell::Open(id),
            None => Cell::Empty,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export, export_to = "Protocol.ts"))]
pub enum GridQuizPhase {
    /// Pre-`StartGame`; players joining.
    Lobby,
    /// `active_picker` chooses a cell.
    BoardSelect,
    /// Question on screen. `floored_player == None` = buzz open; `Some` =
    /// answering. The re-buzz-after-wrong loop stays in this phase.
    QuestionOpen,
    /// Correct answer + verdict shown. Human-paced beat (discussion); exits on
    /// mod `Next` or an optional auto-advance — not auto-timed by default.
    Reveal,
    /// Terminal: board exhausted or mod ended early.
    GameOver,
}

/// The cell in play + its resolved question id. Question content itself lives
/// in the `Dataset`; projection looks it up by id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentCell {
    pub category: usize,
    pub point: usize,
    pub question_id: String,
}

/// One entry in the append-only judgment log. Revising a ruling = append a new
/// entry that supersedes the old, refold, rebroadcast.
#[derive(Clone, Debug, PartialEq)]
pub struct Judgment {
    /// Index into `game.games` — which chain entry (GameEntry) this belongs to.
    /// Equals `current_game_idx` at append time.
    pub game_idx: usize,
    pub player: Token,
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

/// linear play state — not yet designed. Stub so `ModeState` carries both
/// variants from the start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LinearState;
