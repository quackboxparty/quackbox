use crate::data::test_helpers::*;

#[test]
fn catches_pack_referencing_unknown_question() {
    let ds = load(&with_registries(&[(
        "packs/ghost.yaml",
        "id: pack_ghost\ntitle: Ghost\nquestions: [q_does_not_exist]\n",
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("unknown question")));
}

#[test]
fn catches_pack_includes_unknown_pack() {
    let ds = load(&with_registries(&[(
        "packs/a.yaml",
        "id: pack_a\ntitle: A\nincludes: [pack_missing]\nquestions: []\n",
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("unknown pack")));
}

#[test]
fn catches_replaced_by_pointing_nowhere() {
    let ds = load(&with_registries(&[(
        "questions/a.yaml",
        r#"
- id: q_old
  kind: text
  tags: [subject:geo]
  deprecated: { reason: "gone", replaced_by: q_new }
  content:
    default_lang: en
    prompt: { text: "Old?" }
    answer: Old
    variants: { open: { accepted: ["Old"] } }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("replaced_by")));
}
