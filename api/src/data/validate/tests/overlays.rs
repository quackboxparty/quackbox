use crate::data::test_helpers::*;

#[test]
fn catches_overlay_referencing_unknown_question() {
    let ds = load(&with_registries(&[(
        "i18n/de/questions/ghost.yaml",
        "- id: q_not_real\n  content:\n    prompt: { text: \"Kein Problem?\" }\n",
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("references unknown question"))
    );
}

#[test]
fn catches_overlay_referencing_unknown_game() {
    let ds = load(&with_registries(&[(
        "i18n/de/games/ghost.yaml",
        "id: game_not_real\ntitle: Phantom\n",
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("references unknown game"))
    );
}

#[test]
fn catches_overlay_referencing_unknown_tag() {
    let ds = load(&with_registries(&[(
        "i18n/de/tags/subject.yaml",
        "- id: subject:not_real\n  label: Nicht da\n",
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("references unknown tag"))
    );
}

#[test]
fn catches_game_overlay_with_too_many_entries() {
    let ds = load(&with_registries(&[
        (
            "games/base.yaml",
            r#"
id: game_base
title: Base
description: Base
games:
  - mode: linear
    title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#,
        ),
        (
            "i18n/de/games/base.yaml",
            r#"
id: game_base
games:
  - title: Runde 1
  - title: Runde 2
"#,
        ),
    ]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("has 2 game entries but base has 1"))
    );
}

#[test]
fn catches_game_overlay_board_on_linear_entry() {
    let ds = load(&with_registries(&[
        (
            "games/base.yaml",
            r#"
id: game_base
title: Base
description: Base
games:
  - mode: linear
    title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#,
        ),
        (
            "i18n/de/games/base.yaml",
            r#"
id: game_base
games:
  - board:
      categories:
        - name: Nicht erlaubt
"#,
        ),
    ]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("cannot define board for non-grid mode"))
    );
}

#[test]
fn catches_game_overlay_with_too_many_categories() {
    let ds = load(&with_registries(&[
        (
            "games/base.yaml",
            r#"
id: game_base
title: Base
description: Base
games:
  - mode: grid_quiz
    title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    board:
      points: [100, 200]
      categories:
        - name: Geo
          filter: { tags_any: [subject:geo] }
        - name: History
          filter: { tags_any: [subject:history] }
"#,
        ),
        (
            "i18n/de/games/base.yaml",
            r#"
id: game_base
games:
  - board:
      categories:
        - name: Eins
        - name: Zwei
        - name: Drei
"#,
        ),
    ]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("has 3 categories but base has 2"))
    );
}
