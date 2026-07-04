use crate::data::test_helpers::*;

#[test]
fn loads_valid_question_registry_and_pack() {
    let ds = load(&with_registries(&[
        ("questions/test.yaml", VALID_QUESTION),
        ("packs/alpha.yaml", VALID_PACK),
    ]));
    assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
    assert_eq!(ds.questions.len(), 1);
    assert_eq!(ds.packs.len(), 1);
    assert_eq!(ds.tags.len(), 3);
}

#[test]
fn loads_empty_data_dir_without_errors() {
    let ds = load(&[]);
    assert!(ds.issues.is_empty());
}

#[test]
fn catches_duplicate_question_ids() {
    let ds = load(&with_registries(&[
        ("questions/a.yaml", VALID_QUESTION),
        ("questions/b.yaml", VALID_QUESTION),
    ]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("duplicate question id"))
    );
}

#[test]
fn catches_missing_variants_on_text_question() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_novar
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi?" }
    answer: Hi
    variants: {}
"#,
    )]));
    assert!(!ds.issues.is_empty());
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.to_lowercase().contains("variant"))
    );
}

#[test]
fn catches_multiple_choice_with_no_correct() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_nocorrect
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi?" }
    answer: Hi
    variants:
      multiple_choice:
        choices:
          - { id: a, text: A }
          - { id: b, text: B }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("correct")));
}

#[test]
fn catches_non_contiguous_order_positions() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_jumpy
  kind: order
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Order these." }
    items:
      - { id: a, text: A, position: 1 }
      - { id: c, text: C, position: 3 }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("contiguous")));
}

#[test]
fn catches_duplicate_choice_ids() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_dupchoice
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Dup?" }
    answer: A
    variants:
      multiple_choice:
        choices:
          - { id: same, text: A, correct: true }
          - { id: same, text: B }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("duplicate choice id"))
    );
}

#[test]
fn catches_duplicate_order_item_ids() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_dupord
  kind: order
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Order." }
    items:
      - { id: same, text: A, position: 1 }
      - { id: same, text: B, position: 2 }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("duplicate order item id"))
    );
}

#[test]
fn catches_range_max_not_gt_min() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badrange
  kind: numeric
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Year?" }
    answer: 2000
    variants:
      range: { min: 100, max: 50 }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("max must be greater"))
    );
}

#[test]
fn catches_open_variant_with_no_accepted() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_noaccepted
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi?" }
    answer: Hi
    variants:
      open:
        accepted: []
"#,
    )]));
    assert!(!ds.issues.is_empty());
}

#[test]
fn catches_invalid_question_id() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: bad-id!
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("invalid question id"))
    );
}

#[test]
fn catches_invalid_tag_ref_on_question() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_valid
  kind: text
  tags: ["NOT:valid"]
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("invalid tag ref"))
    );
}

#[test]
fn catches_invalid_choice_id() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badchoice
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants:
      multiple_choice:
        choices:
          - { id: "Bad-Id!", text: A, correct: true }
          - { id: b, text: B }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("invalid slug")));
}

#[test]
fn catches_invalid_order_item_id() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badorder
  kind: order
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Order" }
    items:
      - { id: "UPPER", text: A, position: 1 }
      - { id: b, text: B, position: 2 }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("invalid slug")));
}

#[test]
fn catches_invalid_default_lang() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badlang
  kind: text
  tags: [subject:geo]
  content:
    default_lang: NOPE
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("invalid locale"))
    );
}

#[test]
fn catches_invalid_media_ref_prefix() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badmedia
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Hi"
      media:
        - { kind: image, ref: "ftp://nope.png" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(
        ds.issues
            .iter()
            .any(|i| i.message.contains("media ref must start with"))
    );
}

#[test]
fn catches_invalid_youtube_ref() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badyt
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Hi"
      media:
        - { kind: video, ref: "youtube:ab" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("youtube")));
}

#[test]
fn catches_local_ref_with_dotdot() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_dotdot
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Hi"
      media:
        - { kind: image, ref: "local:../etc/passwd" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(ds.issues.iter().any(|i| i.message.contains("local:")));
}

#[test]
fn accepts_valid_locale_with_region() {
    let ds = load(&with_registries(&[(
        "questions/ok.yaml",
        r#"
- id: q_locale_ok
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en-US
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(
        !ds.issues.iter().any(|i| i.message.contains("locale")),
        "unexpected locale issues: {:?}",
        ds.issues
    );
}

#[test]
fn catches_invalid_source_url() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_badsource
  kind: text
  tags: [subject:geo]
  sources:
    - { url: "not-a-url" }
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(!ds.issues.is_empty());
}

#[test]
fn catches_invalid_source_accessed_date() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_baddate
  kind: text
  tags: [subject:geo]
  sources:
    - { url: "https://example.com", accessed: "not-a-date" }
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: Hi
    variants: { open: { accepted: ["Hi"] } }
"#,
    )]));
    assert!(!ds.issues.is_empty());
}

#[test]
fn catches_negative_tolerance() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_negtol
  kind: numeric
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Year?" }
    answer: 2000
    variants:
      numeric_input: { tolerance: -5 }
"#,
    )]));
    assert!(!ds.issues.is_empty());
}

#[test]
fn catches_only_one_choice() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_onechoice
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Hi" }
    answer: A
    variants:
      multiple_choice:
        choices:
          - { id: a, text: A, correct: true }
"#,
    )]));
    assert!(!ds.issues.is_empty());
}

#[test]
fn catches_only_one_order_item() {
    let ds = load(&with_registries(&[(
        "questions/bad.yaml",
        r#"
- id: q_oneitem
  kind: order
  tags: [subject:geo]
  content:
    default_lang: en
    prompt: { text: "Order" }
    items:
      - { id: a, text: A, position: 1 }
"#,
    )]));
    assert!(!ds.issues.is_empty());
}

// ─── Game loader tests ────────────────────────────────────────────
