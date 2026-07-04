use crate::data::test_helpers::*;

#[test]
fn catches_missing_media_file() {
    let ds = load(&with_registries(&[(
        "questions/a.yaml",
        r#"
- id: q_pic
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Look"
      media:
        - { kind: image, ref: "local:img/flag.png" }
    answer: Red
    variants: { open: { accepted: ["Red"] } }
"#,
    )]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("media file missing")));
}

#[test]
fn catches_media_extension_kind_mismatch() {
    let ds = load(&with_registries(&[
        ("media/img/song.mp3", "audio bytes"),
        (
            "questions/a.yaml",
            r#"
- id: q_pic
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Look"
      media:
        - { kind: image, ref: "local:img/song.mp3" }
    answer: Red
    variants: { open: { accepted: ["Red"] } }
"#,
        ),
    ]));
    assert!(ds
        .issues
        .iter()
        .any(|i| i.message.contains("kind mismatch")));
}

#[test]
fn accepts_video_file_used_as_audio() {
    let ds = load(&with_registries(&[
        ("media/clip/vid.mp4", "video bytes"),
        (
            "questions/a.yaml",
            r#"
- id: q_clip
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Listen"
      media:
        - { kind: audio, ref: "local:clip/vid.mp4" }
    answer: Blue
    variants: { open: { accepted: ["Blue"] } }
"#,
        ),
    ]));
    assert!(
        !ds.issues.iter().any(|i| i.message.contains("media")),
        "unexpected media issues: {:?}",
        ds.issues
    );
}

#[test]
fn catches_oversized_image() {
    let big = "x".repeat(101 * 1024);
    let ds = load(&with_registries(&[
        ("media/img/huge.png", &big),
        (
            "questions/a.yaml",
            r#"
- id: q_big
  kind: text
  tags: [subject:geo]
  content:
    default_lang: en
    prompt:
      text: "Oversized"
      media:
        - { kind: image, ref: "local:img/huge.png" }
    answer: Big
    variants: { open: { accepted: ["Big"] } }
"#,
        ),
    ]));
    assert!(ds.issues.iter().any(|i| i.message.contains("size cap")));
}
