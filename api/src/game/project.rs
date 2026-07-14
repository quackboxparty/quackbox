//! The trust boundary — `project(&GameState, &GrantSet) -> ClientView`.
//!
//! The room broadcasts full-truth `GameState`; each socket runs THIS function
//! once before sending. It fills each optional `ClientView` section only if the
//! grants permit it. The correct answer is stripped SERVER-SIDE here — never
//! sent-then-hidden in the client. This is the one place security is NOT
//! simplified away.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    data::{Correctness, Dataset, GameMode, Media, MediaKind, Question, VariantName},
    game::{
        grants::{Grant, GrantSet},
        state::{Cell, GameState, GridQuizPhase, Token},
    },
    protocol::{
        AnswerView, ChoiceView, ClientView, CorrectnessView, GamemodeView, GridQuizView,
        JudgmentView, MediaSrc, MediaView, OrderPositionView, PlayerView, PromptView, QuestionView,
        VariantView,
    },
};

use super::state::ModeState;

pub fn project(data: &Dataset, gamestate: &GameState, grants: &GrantSet) -> ClientView {
    let superseded: HashSet<usize> = gamestate
        .judgment_log
        .iter()
        .filter_map(|judgement| judgement.supersedes)
        .collect();

    let mut scores: HashMap<&Token, i32> = HashMap::new();
    for (i, judgement) in gamestate.judgment_log.iter().enumerate() {
        if !superseded.contains(&i) {
            *scores.entry(&judgement.player).or_default() += judgement.points;
        }
    }

    let players: BTreeMap<String, PlayerView> = gamestate
        .player_slots
        .iter()
        .map(|(token, slot)| {
            (
                slot.name.clone(),
                PlayerView {
                    grants: slot.grants.clone(),
                    score: scores.get(token).copied().unwrap_or(0),
                    connected: slot.connected,
                },
            )
        })
        .collect();

    let judgment_log = gamestate
        .judgment_log
        .iter()
        .map(|judgment| JudgmentView {
            game_idx: judgment.game_idx,
            player: gamestate
                .player_slots
                .name_for_token(&judgment.player)
                .unwrap_or_default(),
            question_id: judgment.question_id.clone(),
            submission: judgment.submission.clone(),
            verdict: judgment.verdict,
            points: judgment.points,
            supersedes: judgment.supersedes,
        })
        .collect();

    let stage = match &gamestate.mode {
        ModeState::GridQuiz(grid_quiz) => {
            let board = match &gamestate.game_config.games[gamestate.current_game_idx].mode {
                GameMode::GridQuiz(game) => &game.board,
                _ => unreachable!("ModeState/GridQuiz mismatch"),
            };

            let used: Vec<Vec<bool>> = grid_quiz
                .cells
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| match cell {
                            Cell::Open(_) => false,
                            _ => true,
                        })
                        .collect()
                })
                .collect();

            GamemodeView::GridQuiz(GridQuizView {
                phase: grid_quiz.phase,
                categories: board
                    .categories
                    .iter()
                    .map(|cat| cat.name.clone())
                    .collect(),
                points: board.points.clone(),
                used,
                active_picker: grid_quiz
                    .active_picker
                    .as_ref()
                    .and_then(|token| gamestate.player_slots.name_for_token(token)),
                floored: grid_quiz
                    .floored_player
                    .as_ref()
                    .and_then(|token| gamestate.player_slots.name_for_token(token)),
                locked_out: grid_quiz
                    .locked_out
                    .iter()
                    .flat_map(|token| gamestate.player_slots.name_for_token(token))
                    .collect(),
            })
        }

        ModeState::Linear(_) => todo!("Linear not implemented yet"),
    };

    let question = match &gamestate.mode {
        ModeState::GridQuiz(grid_quiz) => grid_quiz.current.as_ref().and_then(|cell| {
            let include_answer = grants.contains(&Grant::Moderate)
                || matches!(grid_quiz.phase, GridQuizPhase::Reveal);
            let question = match data.questions.get(&cell.question_id) {
                Some(entry) => &entry.item,
                None => {
                    tracing::warn!(
                        question_id = %cell.question_id,
                        "cell references unknown question id; projecting without question"
                    );
                    return None;
                }
            };
            // TODO: we currently default to the open variant, needs to be fixed when todo 12 lands
            // TODO: we need to translate these fields for the locale the user has
            build_question_view(question, VariantName::Open, include_answer)
        }),
        ModeState::Linear(_) => todo!("Linedar not implemented yet"),
    };

    ClientView {
        players,
        stage,
        question,
        judgment_log,
    }
}

pub(crate) fn build_question_view(
    question: &Question,
    variant: VariantName,
    include_answer: bool,
) -> Option<QuestionView> {
    let variant_view = match variant {
        VariantName::MultipleChoice => {
            let mc = question.mc_choices()?;
            VariantView::MultipleChoice {
                choices: mc
                    .choices
                    .iter()
                    .map(|c| ChoiceView {
                        id: c.id.clone(),
                        text: c.text.clone(),
                        media: c.media.as_deref().and_then(project_first_media),
                    })
                    .collect(),
            }
        }
        VariantName::Open => VariantView::Open,
        VariantName::TrueFalse => VariantView::TrueFalse,
        VariantName::NumericInput => VariantView::NumericInput,
        VariantName::Range => {
            let r = question.range()?;
            VariantView::Range {
                min: r.min,
                max: r.max,
                step: r.step,
            }
        }
    };

    let answer = if include_answer {
        question.correctness(variant).map(|c| AnswerView {
            correctness: c.into(),
            explanation: question.explanation().map(str::to_owned),
        })
    } else {
        None
    };

    let prompt = question.prompt();

    Some(QuestionView {
        prompt: PromptView {
            text: prompt.text.clone(),
            media: prompt.media.as_deref().and_then(project_first_media),
        },
        variant: variant_view,
        answer,
    })
}

impl From<Correctness> for CorrectnessView {
    fn from(c: Correctness) -> Self {
        match c {
            Correctness::MultipleChoice { correct_ids } => Self::MultipleChoice { correct_ids },
            Correctness::Open { accepted } => Self::Open { accepted },
            Correctness::TrueFalse { correct } => Self::TrueFalse { correct },
            Correctness::Numeric { value, tolerance } => Self::Numeric { value, tolerance },
            Correctness::Order { positions } => Self::Order {
                positions: positions
                    .into_iter()
                    .map(|(id, position)| OrderPositionView { id, position })
                    .collect(),
            },
        }
    }
}

fn project_first_media(ms: &[Media]) -> Option<MediaView> {
    // TODO: support multiple media
    ms.first().map(project_media)
}

fn project_media(m: &Media) -> MediaView {
    let src = parse_media_src(&m.media_ref);
    match m.kind {
        MediaKind::Image => MediaView::Image {
            src,
            alt: m.alt.clone(),
            width: m.width,
            height: m.height,
        },
        MediaKind::Video => MediaView::Video {
            src,
            alt: m.alt.clone(),
            width: m.width,
            height: m.height,
            duration_ms: m.duration_ms,
            start_ms: m.start_ms,
            end_ms: m.end_ms,
        },
        MediaKind::Audio => MediaView::Audio {
            src,
            alt: m.alt.clone(),
            duration_ms: m.duration_ms,
            start_ms: m.start_ms,
            end_ms: m.end_ms,
        },
    }
}

fn parse_media_src(r: &str) -> MediaSrc {
    if let Some(id) = r.strip_prefix("youtube:") {
        MediaSrc::Youtube(id.split('?').next().unwrap_or(id).to_string())
    } else if let Some(p) = r.strip_prefix("local:") {
        MediaSrc::Url(format!("/media/{p}"))
    } else if let Some(url) = r.strip_prefix("url:") {
        MediaSrc::Url(url.to_string())
    } else {
        // ponytail: malformed ref passes garde validation only if the dataset
        // was loaded; if a bad ref leaks in, surface as opaque Url for the
        // frontend rather than panicking the projection path.
        MediaSrc::Url(r.to_string())
    }
}

#[cfg(test)]
mod tests {
    //! Trust-boundary guard: `include_answer=false` is the single knob that
    //! strips correctness for Play-only views. Each test constructs a question
    //! with every kind-specific correct-answer field populated, then asserts
    //! that view.answer is None — if any future change leaks a field, this fails.

    use super::*;

    fn load_q(s: &str) -> Question {
        serde_yaml::from_str(s).expect("test fixture must parse")
    }

    const TEXT_Q: &str = r#"
kind: text
id: q_test
tags: []
content:
  default_lang: en
  prompt: { text: "Q" }
  answer: A
  variants:
    multiple_choice:
      choices:
        - { id: a, text: A, correct: true }
        - { id: b, text: B }
    open:
      accepted: ["A"]
"#;

    const NUMERIC_Q: &str = r#"
kind: numeric
id: q_n
tags: []
content:
  default_lang: en
  prompt: { text: "N" }
  answer: 42
  variants:
    numeric_input: { tolerance: 0 }
"#;

    const ORDER_Q: &str = r#"
kind: order
id: q_o
tags: []
content:
  default_lang: en
  prompt: { text: "O" }
  items:
    - { id: x, text: X, position: 1 }
    - { id: y, text: Y, position: 2 }
"#;

    const TF_Q: &str = r#"
kind: text
id: q_tf
tags: []
content:
  default_lang: en
  prompt: { text: "The sky is blue." }
  answer: "true"
  variants:
    true_false: { correct: true }
"#;

    #[test]
    fn text_strips_answer_when_disabled() {
        let q = load_q(TEXT_Q);
        let view = build_question_view(&q, VariantName::Open, false).unwrap();
        assert!(
            view.answer.is_none(),
            "Play view leaked answer: {:?}",
            view.answer
        );
    }

    #[test]
    fn numeric_strips_answer_when_disabled() {
        let q = load_q(NUMERIC_Q);
        let view = build_question_view(&q, VariantName::NumericInput, false).unwrap();
        assert!(
            view.answer.is_none(),
            "Play view leaked answer: {:?}",
            view.answer
        );
    }

    #[test]
    fn order_strips_answer_when_disabled() {
        let q = load_q(ORDER_Q);
        let view = build_question_view(&q, VariantName::Open, false).unwrap();
        assert!(
            view.answer.is_none(),
            "Play view leaked answer: {:?}",
            view.answer
        );
    }

    #[test]
    fn include_answer_true_returns_answer() {
        // Sanity: the gate's other side actually populates the field.
        let q = load_q(TEXT_Q);
        assert!(
            build_question_view(&q, VariantName::Open, true)
                .unwrap()
                .answer
                .is_some()
        );
    }

    #[test]
    fn true_false_strips_answer_when_disabled() {
        let q = load_q(TF_Q);
        let view = build_question_view(&q, VariantName::TrueFalse, false).unwrap();
        assert!(
            view.answer.is_none(),
            "Play view leaked answer: {:?}",
            view.answer
        );
    }

    #[test]
    fn true_false_reveal_carries_correct() {
        let q = load_q(TF_Q);
        let answer = build_question_view(&q, VariantName::TrueFalse, true)
            .unwrap()
            .answer
            .expect("reveal view must include answer");
        assert_eq!(
            answer.correctness,
            CorrectnessView::TrueFalse { correct: true }
        );
    }
}
