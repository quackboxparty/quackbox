use crate::data::test_helpers::*;

#[test]
fn catches_grid_game_referencing_unknown_pack_or_question() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_bad_refs
title: Bad
description: Bad refs
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
        - name: Missing pack
          pack_ref: pack_missing
        - name: Missing question
          question_ids: { 100: q_missing_1, 200: q_missing_2 }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("unknown pack")));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("unknown question"))
    );
}

#[test]
fn catches_linear_game_referencing_unknown_pack() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_bad_linear
title: Bad
description: Bad refs
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
      source: pack
      pack_id: pack_missing
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("unknown pack")));
}

#[test]
fn catches_game_difficulty_map_unknown_tag() {
    let ds = load(&with_registries(&[(
        "games/bad.yaml",
        r#"
id: game_bad_diff
title: Bad
description: Bad diff
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
      difficulty_map:
        100: [subject:not_real]
      categories:
        - name: Geo
          filter: { tags_any: [subject:geo] }
        - name: History
          filter: { tags_any: [subject:history] }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("difficulty_map") && i.message.contains("unknown tag"))
    );
}
