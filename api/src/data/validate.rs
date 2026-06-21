//! Cross-file validation that can't be expressed inside a single YAML schema.
//!
//! Mirrors `validate.ts`: tag refs, pack refs, overlay refs, pack cycles,
//! and media file checks. Returns issues without failing — we accumulate.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::types::*;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const IMAGE_CAP: u64 = 100 * KB;
const MEDIA_CAP: u64 = 1 * MB;

/// Extension → media kind mapping.
fn ext_kind(ext: &str) -> Option<MediaKind> {
    match ext {
        "avif" | "gif" | "jpeg" | "jpg" | "png" | "svg" | "webp" => Some(MediaKind::Image),
        "flac" | "m4a" | "mp3" | "ogg" | "opus" | "wav" => Some(MediaKind::Audio),
        "mov" | "mp4" | "webm" => Some(MediaKind::Video),
        _ => None,
    }
}

/// Run all cross-file checks, returning accumulated issues.
pub fn run_cross_file_checks(ds: &LoadedDataset) -> Vec<LoadIssue> {
    let mut issues = Vec::new();
    check_tag_refs(ds, &mut issues);
    check_refs(ds, &mut issues);
    check_overlay_refs(ds, &mut issues);
    check_pack_cycles(ds, &mut issues);
    check_media_files(ds, &mut issues);
    issues
}

// ─── Tag references ─────────────────────────────────────────────────────────

fn check_tag_refs(ds: &LoadedDataset, issues: &mut Vec<LoadIssue>) {
    for entry in ds.questions.values() {
        for tag in entry.item.tags() {
            if !ds.tags.contains_key(tag) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("unknown tag '{tag}' on question '{}'", entry.item.id()),
                    path: None,
                });
            }
        }
    }

    for entry in ds.packs.values() {
        if let Some(ref f) = entry.item.filter {
            let all_tags = f.tags_all.iter().flatten()
                .chain(f.tags_any.iter().flatten())
                .chain(f.tags_none.iter().flatten());
            for tag in all_tags {
                if !ds.tags.contains_key(tag) {
                    issues.push(LoadIssue {
                        file: entry.file.clone(),
                        message: format!("unknown tag '{tag}' on pack '{}'", entry.item.id),
                        path: None,
                    });
                }
            }
        }
    }
}

// ─── Pack/question references ───────────────────────────────────────────────

fn check_refs(ds: &LoadedDataset, issues: &mut Vec<LoadIssue>) {
    for entry in ds.packs.values() {
        let pack = &entry.item;
        for qid in pack.questions.iter().flatten() {
            if !ds.questions.contains_key(qid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("pack '{}' references unknown question '{qid}'", pack.id),
                    path: None,
                });
            }
        }
        for pid in pack.includes.iter().flatten() {
            if !ds.packs.contains_key(pid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("pack '{}' includes unknown pack '{pid}'", pack.id),
                    path: None,
                });
            }
        }
    }

    for entry in ds.questions.values() {
        if let Some(ref dep) = entry.item.base().deprecated {
            if let Some(ref replaced_by) = dep.replaced_by {
                if !ds.questions.contains_key(replaced_by) {
                    issues.push(LoadIssue {
                        file: entry.file.clone(),
                        message: format!(
                            "question '{}' replaced_by unknown question '{replaced_by}'",
                            entry.item.id()
                        ),
                        path: None,
                    });
                }
            }
        }
    }
}

// ─── Overlay references ─────────────────────────────────────────────────────

fn check_overlay_refs(ds: &LoadedDataset, issues: &mut Vec<LoadIssue>) {
    for (locale, locale_overlays) in &ds.overlays {
        for (qid, entry) in &locale_overlays.questions {
            if !ds.questions.contains_key(qid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' references unknown question '{qid}'"
                    ),
                    path: None,
                });
            }
        }
        for (pid, entry) in &locale_overlays.packs {
            if !ds.packs.contains_key(pid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' references unknown pack '{pid}'"
                    ),
                    path: None,
                });
            }
        }
    }
}

// ─── Pack include cycles (DFS) ──────────────────────────────────────────────

fn check_pack_cycles(ds: &LoadedDataset, issues: &mut Vec<LoadIssue>) {
    let graph: HashMap<&str, &[String]> = ds
        .packs
        .values()
        .map(|e| {
            let includes: &[String] = match &e.item.includes {
                Some(v) => v.as_slice(),
                None => &[],
            };
            (e.item.id.as_str(), includes)
        })
        .collect();

    let file_by_id: HashMap<&str, &str> = ds
        .packs
        .values()
        .map(|e| (e.item.id.as_str(), e.file.as_str()))
        .collect();

    let mut visited = HashSet::new();
    let mut in_progress = HashSet::new();

    for id in graph.keys() {
        dfs_cycle(
            id,
            &graph,
            &file_by_id,
            &mut visited,
            &mut in_progress,
            &mut Vec::new(),
            issues,
        );
    }
}

fn dfs_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, &'a [String]>,
    file_by_id: &HashMap<&'a str, &'a str>,
    visited: &mut HashSet<&'a str>,
    in_progress: &mut HashSet<&'a str>,
    stack: &mut Vec<&'a str>,
    issues: &mut Vec<LoadIssue>,
) {
    if visited.contains(node) {
        return;
    }
    if in_progress.contains(node) {
        let cycle_start = stack.iter().position(|&n| n == node).unwrap_or(0);
        let mut cycle: Vec<&str> = stack[cycle_start..].to_vec();
        cycle.push(node);
        let cycle_str = cycle.join(" -> ");
        issues.push(LoadIssue {
            file: file_by_id.get(node).unwrap_or(&"(unknown)").to_string(),
            message: format!("pack includes cycle: {cycle_str}"),
            path: None,
        });
        return;
    }

    in_progress.insert(node);
    stack.push(node);

    if let Some(deps) = graph.get(node) {
        for next in *deps {
            if graph.contains_key(next.as_str()) {
                dfs_cycle(next, graph, file_by_id, visited, in_progress, stack, issues);
            }
        }
    }

    stack.pop();
    in_progress.remove(node);
    visited.insert(node);
}

// ─── Media file checks ─────────────────────────────────────────────────────

fn check_media_files(ds: &LoadedDataset, issues: &mut Vec<LoadIssue>) {
    let media_dir = Path::new(&ds.data_dir).join("media");

    for entry in ds.questions.values() {
        for (media_ref, kind) in entry.item.media_refs() {
            check_one_media(media_ref, kind, &entry.file, &media_dir, issues);
        }
    }
}

fn check_one_media(
    media_ref: &str,
    kind: MediaKind,
    context_file: &str,
    media_dir: &Path,
    issues: &mut Vec<LoadIssue>,
) {
    let Some(sub) = media_ref.strip_prefix("local:") else {
        return;
    };
    let full = media_dir.join(sub);

    let meta = match fs::metadata(&full) {
        Ok(m) => m,
        Err(_) => {
            issues.push(LoadIssue {
                file: context_file.to_owned(),
                message: format!("media file missing: {media_ref}"),
                path: None,
            });
            return;
        }
    };

    // Extension → kind check
    if let Some(ext) = Path::new(sub).extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if let Some(actual) = ext_kind(&ext_lower) {
            // video-as-audio is allowed (extract audio from video)
            let ok = actual == kind || (actual == MediaKind::Video && kind == MediaKind::Audio);
            if !ok {
                issues.push(LoadIssue {
                    file: context_file.to_owned(),
                    message: format!(
                        "media kind mismatch: declared {kind:?} but extension .{ext_lower} is {actual:?} ({media_ref})"
                    ),
                    path: None,
                });
            }
        } else {
            issues.push(LoadIssue {
                file: context_file.to_owned(),
                message: format!("unknown media extension: .{ext_lower} ({media_ref})"),
                path: None,
            });
        }
    }

    // Size cap
    let cap = if kind == MediaKind::Image { IMAGE_CAP } else { MEDIA_CAP };
    if meta.len() > cap {
        issues.push(LoadIssue {
            file: context_file.to_owned(),
            message: format!(
                "media file exceeds size cap ({}B > {cap}B): {media_ref}",
                meta.len()
            ),
            path: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::data::test_helpers::*;

    // ─── Cross-file ref checks ──────────────────────────────────────

    #[test]
    fn catches_pack_referencing_unknown_question() {
        let ds = load(&with_registries(&[(
            "packs/ghost.yaml",
            "id: pack_ghost\ntitle: Ghost\nquestions: [q_does_not_exist]\n",
        )]));
        assert!(ds.issues.iter().any(|i| i.message.contains("unknown question")));
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

    #[test]
    fn catches_overlay_referencing_unknown_question() {
        let ds = load(&with_registries(&[(
            "i18n/de/questions/ghost.yaml",
            "- id: q_not_real\n  content:\n    prompt: { text: \"Kein Problem?\" }\n",
        )]));
        assert!(ds.issues.iter().any(|i| i.message.contains("references unknown question")));
    }

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
        assert!(ds.issues.iter().any(|i| i.message.contains("media file missing")));
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
        assert!(ds.issues.iter().any(|i| i.message.contains("kind mismatch")));
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
}
