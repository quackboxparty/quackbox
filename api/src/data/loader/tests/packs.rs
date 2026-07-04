use crate::data::test_helpers::*;

#[test]
fn catches_pack_without_content() {
    let ds = load(&with_registries(&[(
        "packs/empty.yaml",
        "id: pack_empty\ntitle: Empty\n",
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("at least one of")));
}

#[test]
fn catches_invalid_pack_id() {
    let ds = load(&with_registries(&[(
        "packs/bad.yaml",
        "id: notapackid\ntitle: Bad\nquestions: [q_alpha_one]\n",
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("invalid pack id")));
}

#[test]
fn catches_invalid_question_ref_in_pack() {
    let ds = load(&with_registries(&[(
        "packs/bad.yaml",
        "id: pack_bad\ntitle: Bad\nquestions: [NOT-VALID]\n",
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("invalid question id")));
}

#[test]
fn catches_invalid_includes_ref_in_pack() {
    let ds = load(&with_registries(&[(
        "packs/bad.yaml",
        "id: pack_bad\ntitle: Bad\nincludes: [not_a_pack_id]\n",
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("invalid pack id")));
}

#[test]
fn catches_invalid_tag_ref_in_pack_filter() {
    let ds = load(&with_registries(&[(
        "packs/bad.yaml",
        "id: pack_bad\ntitle: Bad\nfilter:\n  tags_any: [INVALID]\n",
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("invalid tag ref")));
}
