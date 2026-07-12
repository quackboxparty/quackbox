use crate::data::test_helpers::*;
use crate::data::{BuzzPolicy, GameMode, LinearSource, ScoringMode, StealPolicy};

use super::fixtures::*;

#[test]
fn loads_valid_grid_game_config() {
    let ds = load(&with_registries(&[
        ("questions/test.yaml", VALID_QUESTION),
        ("questions/two.yaml", VALID_QUESTION_TWO),
        ("games/test.yaml", VALID_GRID_GAME),
    ]));
    assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
    assert_eq!(ds.games.len(), 1);
    let gc = ds.games.get("game_test_grid").unwrap();
    assert_eq!(gc.item.games.len(), 1);
    assert_eq!(gc.item.title, "Test Grid");
    let game = &gc.item.games[0];
    match &game.mode {
        GameMode::GridQuiz(g) => {
            assert_eq!(game.title, "Round 1");
            assert!(matches!(game.rules.buzz_policy, BuzzPolicy::OpenFloor));
            assert!(matches!(game.rules.steal_policy, StealPolicy::RoundLimited));
            assert_eq!(g.board.points.len(), 4);
            assert_eq!(g.board.categories.len(), 2);
            assert_eq!(g.board.categories[1].name, "Flags");
        }
        _ => panic!("expected GridQuiz"),
    }
}

#[test]
fn loads_valid_linear_game_config() {
    let ds = load(&with_registries(&[
        ("questions/test.yaml", VALID_QUESTION),
        ("games/test.yaml", VALID_LINEAR_GAME),
    ]));
    assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
    assert_eq!(ds.games.len(), 1);
    let gc = ds.games.get("game_test_linear").unwrap();
    let game = &gc.item.games[0];
    match &game.mode {
        GameMode::Linear(g) => {
            assert!(matches!(game.rules.buzz_policy, BuzzPolicy::Broadcast));
            assert!(matches!(game.rules.scoring_mode, ScoringMode::AllGrade));
            assert!(matches!(game.rules.steal_policy, StealPolicy::None));
            match &g.questions {
                LinearSource::Questions { question_ids } => {
                    assert_eq!(question_ids.as_slice(), &["q_alpha_one"]);
                }
                _ => panic!("expected Questions source"),
            }
        }
        _ => panic!("expected Linear"),
    }
}

#[test]
fn loads_game_with_default_timers() {
    let ds = load(&with_registries(&[(
        "games/test.yaml",
        r#"
id: game_short
title: Short
description: Uses default timers
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        categories:
          - name: Geo
            filter: { tags_any: [subject:geo] }
          - name: History
            filter: { tags_any: [subject:history] }
"#,
    )]));
    assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
    let gc = ds.games.get("game_short").unwrap();
    let game = &gc.item.games[0];
    match &game.mode {
        GameMode::GridQuiz(_) => {
            assert_eq!(game.rules.question_timer_secs, 30); // default
            assert_eq!(game.rules.answer_timer_secs, 15); // default
        }
        _ => panic!("expected GridQuiz"),
    }
}

#[test]
fn catches_duplicate_game_ids() {
    let dup = r#"
id: game_dup
title: A
description: A
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        categories:
          - name: Math
            filter: { tags_any: [subject:math] }
          - name: Geo
            filter: { tags_any: [subject:geo] }
"#;
    let ds = load(&[("games/a.yaml", dup), ("games/b.yaml", dup)]);
    assert!(ds.issues.iter().any(|i| i.message.contains("duplicate")));
}

#[test]
fn game_entry_validates_broadcast_first_correct() {
    let ds = load(&[(
        "games/bad.yaml",
        r#"
id: game_bad_combo
title: Bad
description: Bad combo
games:
  - title: R1
    rules:
      buzz_policy: broadcast
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: linear
      questions:
        source: questions
        question_ids: [q_alpha_one]
"#,
    )]);
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("broadcast") && i.message.to_lowercase().contains("first"))
    );
}

#[test]
fn game_entry_validates_broadcast_steal() {
    let ds = load(&[(
        "games/bad.yaml",
        r#"
id: game_bad_steal
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: broadcast
      scoring_mode: all_grade
      lockout_policy: none
      steal_policy: open_floor
      judge: auto
    mode:
      kind: linear
      questions:
        source: questions
        question_ids: [q_alpha_one]
"#,
    )]);
    assert!(ds.issues.iter().any(|i| i.message.contains("steal_policy")));
}

#[test]
fn loads_board_with_explicit_question_ids() {
    let ds = load(&[("games/test.yaml", VALID_GRID_GAME)]);
    let gc = ds.games.get("game_test_grid").unwrap();
    let game = &gc.item.games[0];
    match &game.mode {
        GameMode::GridQuiz(g) => {
            let flags = &g.board.categories[1];
            let ids = flags.question_ids.as_ref().unwrap();
            assert_eq!(ids.get(&100), Some(&"q_alpha_one".to_string()));
            assert_eq!(ids.get(&200), Some(&"q_alpha_two".to_string()));
        }
        _ => panic!("expected GridQuiz"),
    }
}

#[test]
fn catches_game_with_single_category() {
    let ds = load(&[(
        "games/bad.yaml",
        r#"
id: game_onetop
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        categories:
          - name: Only One
            filter: { tags_any: [subject:math] }
"#,
    )]);
    assert!(!ds.issues.is_empty());
}

#[test]
fn catches_invalid_game_id() {
    let ds = load(&[(
        "games/bad.yaml",
        r#"
id: not_a_game_id
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        categories:
          - name: Math
            filter: { tags_any: [subject:math] }
          - name: Geo
            filter: { tags_any: [subject:geo] }
"#,
    )]);
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("invalid game id"))
    );
}

#[test]
fn rejects_unknown_game_field() {
    let ds = load(&[(
        "games/bad.yaml",
        r#"
id: game_unknown_field
title: Bad
description: Bad
oops: true
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: linear
      questions:
        source: questions
        question_ids: [q_alpha_one]
"#,
    )]);
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("unknown field") && i.message.contains("oops"))
    );
}

#[test]
fn catches_zero_timers_on_game_rules() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_zero_timer
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
      question_timer_secs: 0
      answer_timer_secs: 0
    mode:
      kind: linear
      questions:
        source: questions
        question_ids: [q_alpha_one]
"#,
    )]));
    assert!(ds.issues.iter().any(|i| {
        i.message
            .contains("question_timer_secs must be greater than 0")
    }));
}

#[test]
fn catches_grid_points_not_strictly_increasing() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_bad_points
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [200, 100]
        categories:
          - name: Geo
            filter: { tags_any: [subject:geo] }
          - name: History
            filter: { tags_any: [subject:history] }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| {
        i.message
            .contains("board points must be strictly increasing")
    }));
}

#[test]
fn catches_grid_question_ids_key_not_in_points() {
    let ds = load(&with_registries(&[
        ("questions/test.yaml", VALID_QUESTION),
        (
            "games/bad.yaml",
            r#"
id: game_bad_keys
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        categories:
          - name: Geo
            question_ids: { 300: q_alpha_one }
          - name: History
            filter: { tags_any: [subject:history] }
"#,
        ),
    ]));
    assert!(ds.issues.iter().any(|i| {
        i.message
            .contains("question_ids key 300 must be present in board.points")
    }));
}

#[test]
fn catches_grid_duplicate_explicit_question_ids() {
    let ds = load(&with_registries(&[
        ("questions/test.yaml", VALID_QUESTION),
        (
            "games/bad.yaml",
            r#"
id: game_dup_qids
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        categories:
          - name: Geo
            question_ids: { 100: q_alpha_one }
          - name: History
            question_ids: { 200: q_alpha_one }
"#,
        ),
    ]));
    assert!(ds.issues.iter().any(|i| {
        i.message
            .contains("explicit question id 'q_alpha_one' is duplicated")
    }));
}

#[test]
fn catches_linear_duplicate_question_ids() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_dup_linear
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: linear
      questions:
        source: questions
        question_ids: [q_alpha_one, q_alpha_one]
"#,
    )]));
    assert!(ds.issues.iter().any(|i| {
        i.message
            .contains("has duplicate question id 'q_alpha_one'")
    }));
}

#[test]
fn catches_difficulty_map_key_not_in_points() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_bad_diff_key
title: Bad
description: Bad
games:
  - title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    mode:
      kind: grid_quiz
      board:
        points: [100, 200]
        difficulty_map:
          300: [subject:geo]
        categories:
          - name: Geo
            filter: { tags_any: [subject:geo] }
          - name: History
            filter: { tags_any: [subject:history] }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| {
        i.message
            .contains("difficulty_map key 300 must be present in board.points")
    }));
}
