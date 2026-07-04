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
    check_game_refs(ds, &mut issues);
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

// ─── Game references ───────────────────────────────────────────────────────

fn check_game_refs(ds: &LoadedDataset, issues: &mut Vec<LoadIssue>) {
    for entry in ds.games.values() {
        let gc = &entry.item;
        for (game_idx, game_entry) in gc.games.iter().enumerate() {
            match game_entry {
                GameEntry::GridQuiz(g) => {
                    for (cat_idx, cat) in g.board.categories.iter().enumerate() {
                        let ctx = format!("game '{}' entry[{game_idx}] category[{cat_idx}]", gc.id);

                        if let Some(pack_id) = &cat.pack_ref
                            && !ds.packs.contains_key(pack_id)
                        {
                            issues.push(LoadIssue {
                                file: entry.file.clone(),
                                message: format!("{ctx} references unknown pack '{pack_id}'"),
                                path: None,
                            });
                        }

                        for qid in cat.question_ids.iter().flat_map(|map| map.values()) {
                            if !ds.questions.contains_key(qid) {
                                issues.push(LoadIssue {
                                    file: entry.file.clone(),
                                    message: format!(
                                        "{ctx} references unknown question '{qid}'"
                                    ),
                                    path: None,
                                });
                            }
                        }

                        if let Some(filter) = &cat.filter {
                            check_filter_tags(ds, filter, &entry.file, &ctx, issues);
                        }
                    }

                    if let Some(diff_map) = &g.board.difficulty_map {
                        for (point, tags) in diff_map {
                            for tag in tags {
                                if !ds.tags.contains_key(tag) {
                                    issues.push(LoadIssue {
                                        file: entry.file.clone(),
                                        message: format!(
                                            "unknown tag '{tag}' on game '{}' entry[{game_idx}] difficulty_map[{point}]",
                                            gc.id
                                        ),
                                        path: None,
                                    });
                                }
                            }
                        }
                    }
                }
                GameEntry::Linear(g) => match &g.questions {
                    LinearSource::Questions { question_ids } => {
                        for qid in question_ids {
                            if !ds.questions.contains_key(qid) {
                                issues.push(LoadIssue {
                                    file: entry.file.clone(),
                                    message: format!(
                                        "game '{}' entry[{game_idx}] references unknown question '{qid}'",
                                        gc.id
                                    ),
                                    path: None,
                                });
                            }
                        }
                    }
                    LinearSource::Pack { pack_id } => {
                        if !ds.packs.contains_key(pack_id) {
                            issues.push(LoadIssue {
                                file: entry.file.clone(),
                                message: format!(
                                    "game '{}' entry[{game_idx}] references unknown pack '{pack_id}'",
                                    gc.id
                                ),
                                path: None,
                            });
                        }
                    }
                    LinearSource::Filter { filter } => {
                        let ctx = format!("game '{}' entry[{game_idx}] linear filter", gc.id);
                        check_filter_tags(ds, filter, &entry.file, &ctx, issues);
                    }
                },
            }
        }
    }
}

fn check_filter_tags(
    ds: &LoadedDataset,
    filter: &PackFilter,
    file: &str,
    context: &str,
    issues: &mut Vec<LoadIssue>,
) {
    let all_tags = filter
        .tags_all
        .iter()
        .flatten()
        .chain(filter.tags_any.iter().flatten())
        .chain(filter.tags_none.iter().flatten());

    for tag in all_tags {
        if !ds.tags.contains_key(tag) {
            issues.push(LoadIssue {
                file: file.to_owned(),
                message: format!("unknown tag '{tag}' on {context}"),
                path: None,
            });
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
        for (tid, entry) in &locale_overlays.tags {
            if !ds.tags.contains_key(tid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("overlay of locale '{locale}' references unknown tag '{tid}'"),
                    path: None,
                });
            }
        }
        for (gid, entry) in &locale_overlays.games {
            let Some(base_game) = ds.games.get(gid) else {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' references unknown game '{gid}'"
                    ),
                    path: None,
                });
                continue;
            };

            if entry.item.games.len() > base_game.item.games.len() {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' game '{gid}' has {} game entries but base has {}",
                        entry.item.games.len(),
                        base_game.item.games.len()
                    ),
                    path: None,
                });
            }

            for (idx, ovl_game) in entry.item.games.iter().enumerate() {
                let Some(base_entry) = base_game.item.games.get(idx) else {
                    continue;
                };

                if let Some(board_ovl) = &ovl_game.board {
                    match base_entry {
                        GameEntry::GridQuiz(base_grid) => {
                            if board_ovl.categories.len() > base_grid.board.categories.len() {
                                issues.push(LoadIssue {
                                    file: entry.file.clone(),
                                    message: format!(
                                        "overlay of locale '{locale}' game '{gid}' entry[{idx}] has {} categories but base has {}",
                                        board_ovl.categories.len(),
                                        base_grid.board.categories.len()
                                    ),
                                    path: None,
                                });
                            }
                        }
                        GameEntry::Linear(_) => {
                            issues.push(LoadIssue {
                                file: entry.file.clone(),
                                message: format!(
                                    "overlay of locale '{locale}' game '{gid}' entry[{idx}] cannot define board for non-grid mode"
                                ),
                                path: None,
                            });
                        }
                    }
                }
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
        assert!(ds.issues.iter().any(|i| i.message.contains("unknown question")));
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
    fn catches_overlay_referencing_unknown_game() {
        let ds = load(&with_registries(&[(
            "i18n/de/games/ghost.yaml",
            "id: game_not_real\ntitle: Phantom\n",
        )]));
        assert!(ds.issues.iter().any(|i| i.message.contains("references unknown game")));
    }

    #[test]
    fn catches_overlay_referencing_unknown_tag() {
        let ds = load(&with_registries(&[(
            "i18n/de/tags/subject.yaml",
            "- id: subject:not_real\n  label: Nicht da\n",
        )]));
        assert!(ds.issues.iter().any(|i| i.message.contains("references unknown tag")));
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
