use crate::data::test_helpers::*;

use super::fixtures::*;

#[test]
fn rejects_unknown_question_overlay_field() {
    let ds = load(&[(
        "i18n/de/questions/bad.yaml",
        r#"
- id: q_alpha_one
  content:
    prompt: { text: "Hallo" }
    correct: true
"#,
    )]);
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("unknown field") && i.message.contains("correct"))
    );
}

#[test]
fn rejects_unknown_game_overlay_field() {
    let ds = load(&[(
        "i18n/de/games/bad.yaml",
        r#"
id: game_test_grid
title: Test
rules:
  judge: auto
"#,
    )]);
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("unknown field") && i.message.contains("rules"))
    );
}

#[test]
fn game_overlay_loads_without_issues() {
    let ds = load(&with_registries(&[
        ("questions/test.yaml", VALID_QUESTION),
        ("questions/two.yaml", VALID_QUESTION_TWO),
        ("games/test.yaml", VALID_GRID_GAME),
        (
            "i18n/de/games/test.yaml",
            r#"
id: game_test_grid
title: Test Gitter
"#,
        ),
    ]));
    let ovl = ds
        .overlays
        .get("de")
        .and_then(|o| o.games.get("game_test_grid"));
    assert!(ovl.is_some());
    assert_eq!(ovl.unwrap().item.title.as_deref(), Some("Test Gitter"));
}

// ─── Integration───────
