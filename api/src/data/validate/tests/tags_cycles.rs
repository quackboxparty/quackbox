use crate::data::test_helpers::*;

#[test]
fn catches_unknown_tag_on_question() {
    let ds = load(&with_registries(&[(
        "questions/a.yaml",
        r#"
- id: q_taggy
  kind: text
  tags: [subject:nonexistent]
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("unknown tag")));
}

#[test]
fn catches_pack_includes_cycle() {
    let ds = load(&with_registries(&[
        (
            "packs/a.yaml",
            "id: pack_a\ntitle: A\nincludes: [pack_b]\nquestions: []\n",
        ),
        (
            "packs/b.yaml",
            "id: pack_b\ntitle: B\nincludes: [pack_a]\nquestions: []\n",
        ),
    ]));
    assert!(ds.issues.iter().any(|i| i.message.contains("cycle")));
}

// ─── Media checks ───────────────────────────────────────────────
