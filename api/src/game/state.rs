//! `GameState` — the full in-memory truth for one room: players, scores
//! (folded from the judgment log, never a mutated counter), current question,
//! timer DEADLINE timestamps (not countdowns), phase. Must be `Clone` (each
//! broadcast subscriber gets a copy).
//!
//! `snapshot()` produces the full-truth value the room broadcasts; per-role
//! stripping happens later in `project`, not here.
//!
//! TODO: GameState, apply(Command), on_timeout, snapshot, score = fold(log).

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::{Deref, DerefMut},
};

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
        state::GridQuizPhase::BoardSelect,
    },
    protocol::Command,
};

#[derive(Clone, Debug)]
pub struct GameState {
    pub game_config: GameConfig,
    /// Which entry in the game chain (`game.games[..]`) is currently live.
    pub current_game_idx: usize,
    pub player_slots: PlayerSlots,
    pub mode: ModeState,
    /// Global, append-only, spans all rounds. `score = fold(judgment_log)`.
    pub judgment_log: Vec<Judgment>,
}

impl GameState {
    pub fn apply(&mut self, token: Token, cmd: Command) {
        if let Some(needed) = cmd.required_grant() {
            let ok = self
                .player_slots
                .grants_for(&token)
                .is_some_and(|grants| grants.contains(&needed));
            if !ok {
                tracing::info!(?token, ?needed, "command without required grant");
                return;
            }
        }

        match cmd {
            Command::Kick { player } => {
                self.player_slots.retain(|_, v| v.name != player);
            }
            other => {
                for effect in self.mode.apply(&self.player_slots, token, other) {
                    self.run_effect(effect);
                }
            }
        }
    }

    fn run_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Submit {
                player,
                question_id,
                text,
            } => {
                if let Some(idx) = self.live_judgment(&player, &question_id) {
                    if self.judgment_log[idx].verdict == Verdict::Pending {
                        // TODO: maybe we want to allow updating the answer before judgment
                        tracing::warn!(?player, "answer while pending judgment");
                        return;
                    }
                }

                self.judgment_log.push(Judgment {
                    game_idx: self.current_game_idx,
                    player,
                    points: 0,
                    verdict: Verdict::Pending,
                    question_id,
                    submission: Some(text),
                    supersedes: None,
                });
            }
            Effect::Rule {
                target,
                question_id,
                verdict,
                points,
            } => {
                let pending_idx = self.live_judgment(&target, &question_id);
                self.judgment_log.push(Judgment {
                    player: target,
                    question_id,
                    verdict,
                    points,
                    game_idx: self.current_game_idx,
                    submission: None,
                    supersedes: pending_idx,
                });
            }
        }
    }

    // TODO: rebuilds the superseded set on every call, O(n) per lookup. fine at
    // quiz-room scale. if the log grows large, cache it or store a superseded flag
    // on each entry.
    fn live_judgment(&self, player: &Token, question_id: &str) -> Option<usize> {
        let superseded: HashSet<usize> = self
            .judgment_log
            .iter()
            .filter_map(|j| j.supersedes)
            .collect();
        self.judgment_log
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, j)| {
                (j.player == *player && j.question_id == question_id && !superseded.contains(&i))
                    .then_some(i)
            })
    }

    pub(crate) fn current_game(&self) -> &Game {
        self.game_config
            .games
            .get(self.current_game_idx)
            .expect("current game idx does not yield a game")
    }
}

#[derive(Clone, Debug)]
pub struct PlayerSlots(HashMap<Token, PlayerSlot>);

impl PlayerSlots {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn grants_for(&self, token: &Token) -> Option<&GrantSet> {
        match self.get(token) {
            Some(slot) => Some(&slot.grants),
            None => None,
        }
    }

    pub(crate) fn name_for_token(&self, token: &Token) -> Option<String> {
        match self.get(token) {
            Some(slot) => Some(slot.name.clone()),
            None => None,
        }
    }

    pub(crate) fn token_for_name(&self, name: &str) -> Option<Token> {
        self.iter()
            .find(|(_, slot)| slot.name == name)
            .map(|(token, _)| token.clone())
    }
}

impl Default for PlayerSlots {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for PlayerSlots {
    type Target = HashMap<Token, PlayerSlot>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for PlayerSlots {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

enum Effect {
    Submit {
        player: Token,
        question_id: String,
        text: String,
    },
    Rule {
        target: Token,
        question_id: String,
        verdict: Verdict,
        points: i32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModeState {
    GridQuiz(GridQuizState),
    Linear(LinearState),
}

impl ModeState {
    fn apply(&mut self, player_slots: &PlayerSlots, token: Token, cmd: Command) -> Vec<Effect> {
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

                    modestate.phase = GridQuizPhase::QuestionOpen;
                }
                Command::Answer { text } => {
                    if modestate.floored_player.as_ref() != Some(&token)
                        && player_slots
                            .grants_for(&token)
                            .is_none_or(|grants| !grants.contains(&Grant::Moderate))
                    {
                        tracing::warn!(?token, "tried to answer while not being floored");
                        return Vec::new();
                    }

                    let Some(current) = modestate.current.as_ref() else {
                        tracing::warn!(?token, "answer with no cell in play");
                        return Vec::new();
                    };

                    // TODO: we should check for a pending judgement here before pushing, maybe we
                    // could even allow or prevent updating your answer
                    return vec![Effect::Submit {
                        player: token,
                        question_id: current.question_id.clone(),
                        text,
                    }];
                }
                Command::Rule { player, verdict } => {
                    let Some(current) = modestate.current.as_ref() else {
                        tracing::warn!(?token, "rule with no cell in play");
                        return Vec::new();
                    };

                    let Some(&value) = modestate.points.get(current.point) else {
                        tracing::warn!(point = current.point, "rule with out-of-range point");
                        return Vec::new();
                    };
                    let Some(target) = player_slots.token_for_name(&player) else {
                        tracing::warn!(?player, "rule for unknown player");
                        return Vec::new();
                    };

                    let points = match verdict {
                        Verdict::Correct => value as i32,
                        Verdict::Incorrect => -(value as i32) / 2,
                        Verdict::Void | Verdict::Pending => 0,
                    };

                    match verdict {
                        Verdict::Correct | Verdict::Void => {
                            modestate.floored_player = None;
                            modestate.cells[current.category][current.point] =
                                Cell::Used(current.question_id.clone());

                            if modestate.cells.iter().any(|column| {
                                column.iter().any(|cell| matches!(cell, Cell::Open(_)))
                            }) {
                                modestate.phase = GridQuizPhase::Reveal;
                            } else {
                                modestate.phase = GridQuizPhase::GameOver;
                            }
                        }
                        Verdict::Incorrect => {
                            modestate.floored_player = None;
                            // TODO: check if all players are locked out and close question/reveal
                            // automatically maybe
                            modestate.locked_out.insert(target.clone());
                        }
                        _ => {}
                    }

                    return vec![Effect::Rule {
                        target,
                        question_id: current.question_id.clone(),
                        verdict,
                        points,
                    }];
                }
                Command::Buzz => {
                    if let Some(floored_player) = &modestate.floored_player {
                        tracing::warn!(?floored_player, "tried to buzz while player is floored");
                        return Vec::new();
                    }

                    if &modestate.phase != &GridQuizPhase::QuestionOpen {
                        tracing::warn!(?token, "tried to buzz while no question was open");
                        return Vec::new();
                    }

                    modestate.floored_player = Some(token);
                }
                Command::Next => {
                    modestate.locked_out = HashSet::new();
                    modestate.current = None;
                    modestate.active_picker = modestate.picker_rotation.front().cloned();
                    modestate.phase = GridQuizPhase::BoardSelect;
                }
                Command::CloseQuestion => {
                    let Some(current) = modestate.current.as_ref() else {
                        tracing::warn!(?token, "tried to close question with no current cell");
                        return Vec::new();
                    };

                    if &modestate.phase != &GridQuizPhase::QuestionOpen {
                        tracing::warn!(
                            ?token,
                            "tried to close question while no question was open"
                        );
                        return Vec::new();
                    }

                    modestate.floored_player = None;
                    modestate.cells[current.category][current.point] =
                        Cell::Used(current.question_id.clone());

                    if modestate
                        .cells
                        .iter()
                        .any(|column| column.iter().any(|cell| matches!(cell, Cell::Open(_))))
                    {
                        modestate.phase = GridQuizPhase::Reveal;
                    } else {
                        modestate.phase = GridQuizPhase::GameOver;
                    }
                }
                _ => todo!("other gridquiz cmds not implemented yet"),
            },
            ModeState::Linear(_) => todo!("Linear not implemented yet"),
        };
        return Vec::new();
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
    pub points: Vec<u32>,
}

impl GridQuizState {
    pub fn build(cells: Vec<Vec<Cell>>, points: Vec<u32>) -> Self {
        Self {
            phase: GridQuizPhase::Lobby,
            active_picker: None,
            floored_player: None,
            locked_out: HashSet::new(),
            current: None,
            picker_rotation: VecDeque::new(),
            cells,
            points,
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
