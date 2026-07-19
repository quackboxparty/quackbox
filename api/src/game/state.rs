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
    data::{Game, GameConfig},
    game::{
        grants::{Grant, GrantSet},
        judge::Verdict,
    },
    protocol::Command,
};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("no cell in play")]
    NoCurrentCell,
    #[error("no players in game")]
    NoPlayers,
    #[error("player not floored {0}")]
    PlayerNotFloored(String),
    #[error("point out of range")]
    PointOutOfRange,
    #[error("buzzed while floored player")]
    BuzzWhileFlooredPlayer,
    #[error("question not open (phase {0:?})")]
    WrongPhase(GridQuizPhase),
    #[error("unknown player {0}")]
    UnknownPlayer(String),
    #[error("missing grant {0:?}")]
    MissingGrant(Grant),
}

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
            Command::Grant { player, grants } => {
                let Some(token) = self.player_slots.token_for_name(&player) else {
                    tracing::info!(?player, "grant by unknown player");
                    return;
                };

                self.player_slots
                    .entry(token)
                    .and_modify(|slot| slot.grants = grants);
            }
            other => match self.mode.apply(&self.player_slots, token.clone(), other) {
                Ok(effects) => effects
                    .into_iter()
                    .for_each(|effect| self.run_effect(effect)),
                Err(err) => tracing::warn!(?token, %err, "command rejected"),
            },
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

    pub(crate) fn is_grant(&self, token: &Token, grant: &Grant) -> bool {
        self.grants_for(token)
            .is_some_and(|grants| grants.contains(grant))
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
    fn apply(
        &mut self,
        player_slots: &PlayerSlots,
        token: Token,
        cmd: Command,
    ) -> Result<Vec<Effect>, CommandError> {
        match self {
            ModeState::GridQuiz(modestate) => match cmd {
                Command::StartGame => {
                    // TODO: maybe shuffle this
                    let rotation: VecDeque<Token> = player_slots
                        .iter()
                        .filter(|(player_token, player)| {
                            player.connected
                                && player_slots
                                    .grants_for(player_token)
                                    .is_some_and(|grants| grants.contains(&Grant::Play))
                        })
                        .map(|(token, _)| token.clone())
                        .collect();

                    if rotation.is_empty() {
                        return Err(CommandError::NoPlayers);
                    }

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
                        && !player_slots.is_grant(&token, &Grant::Moderate)
                    {
                        return Err(CommandError::PlayerNotFloored(token.0));
                    }

                    let Some(current) = modestate.current.as_ref() else {
                        return Err(CommandError::NoCurrentCell);
                    };

                    // TODO: we should check for a pending judgement here before pushing, maybe we
                    // could even allow or prevent updating your answer
                    return Ok(vec![Effect::Submit {
                        player: token,
                        question_id: current.question_id.clone(),
                        text,
                    }]);
                }
                Command::Rule { player, verdict } => {
                    let Some(current) = modestate.current.as_ref() else {
                        return Err(CommandError::NoCurrentCell);
                    };

                    let Some(&value) = modestate.points.get(current.point) else {
                        return Err(CommandError::PointOutOfRange);
                    };
                    let Some(target) = player_slots.token_for_name(&player) else {
                        return Err(CommandError::UnknownPlayer(player));
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

                            let all_locked = player_slots
                                .iter()
                                .filter(|(player_token, slot)| {
                                    slot.connected
                                        && player_slots
                                            .grants_for(&player_token)
                                            .is_some_and(|grants| grants.contains(&Grant::Play))
                                })
                                .all(|(player_token, _)| {
                                    modestate.locked_out.contains(player_token)
                                });

                            if all_locked {
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
                        }
                        _ => {}
                    }

                    return Ok(vec![Effect::Rule {
                        target,
                        question_id: current.question_id.clone(),
                        verdict,
                        points,
                    }]);
                }
                Command::Buzz => {
                    if modestate.floored_player.is_some() {
                        return Err(CommandError::BuzzWhileFlooredPlayer);
                    }

                    if &modestate.phase != &GridQuizPhase::QuestionOpen {
                        return Err(CommandError::WrongPhase(modestate.phase));
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
                        return Err(CommandError::NoCurrentCell);
                    };

                    if &modestate.phase != &GridQuizPhase::QuestionOpen {
                        return Err(CommandError::WrongPhase(modestate.phase));
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
        return Ok(Vec::new());
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
    pub(crate) fn build(cells: Vec<Vec<Cell>>, points: Vec<u32>) -> Self {
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

    pub(crate) fn close_current(&mut self) {
        let Some(current) = self.current.as_ref() else {
            tracing::warn!("tried to close question with no current cell");
            return;
        };

        if &self.phase != &GridQuizPhase::QuestionOpen {
            tracing::warn!("tried to close question while no question was open");
            return;
        }

        self.floored_player = None;
        self.cells[current.category][current.point] = Cell::Used(current.question_id.clone());

        if self
            .cells
            .iter()
            .any(|column| column.iter().any(|cell| matches!(cell, Cell::Open(_))))
        {
            self.phase = GridQuizPhase::Reveal;
        } else {
            self.phase = GridQuizPhase::GameOver;
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
