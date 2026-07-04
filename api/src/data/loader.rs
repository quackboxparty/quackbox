//! Walk data directories, parse YAML files, build registries.
//!

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fs;
use std::path::{Path, PathBuf};

use garde::Validate;

use super::error::LoadError;
use super::types::*;

// ─── Per-file parse result ──────────────────────────────────────────────────

enum Parsed<T> {
    Ok {
        file: String,
        items: Vec<T>,
        issues: Vec<LoadIssue>,
    },
    Err {
        issues: Vec<LoadIssue>,
    },
}

/// Parse a YAML file, deserialize via `decode`, optionally run `validate` on each item.
fn parse_file<T, Raw: serde::de::DeserializeOwned>(
    path: &Path,
    rel_path: &str,
    decode: fn(Raw) -> Vec<T>,
    validate: Option<fn(&T, &str) -> Vec<LoadIssue>>,
) -> Parsed<T> {
    let raw = match parse_yaml_file::<Raw>(path) {
        Ok(v) => v,
        Err(e) => {
            return Parsed::Err {
                issues: vec![yaml_issue(rel_path, e)],
            };
        }
    };
    let items = decode(raw);
    let issues = match validate {
        Some(f) => items.iter().flat_map(|item| f(item, rel_path)).collect(),
        None => Vec::new(),
    };
    Parsed::Ok {
        file: rel_path.to_owned(),
        items,
        issues,
    }
}

fn parse_many<T: serde::de::DeserializeOwned>(raw: Vec<T>) -> Vec<T> {
    raw
}
fn parse_one<T>(raw: T) -> Vec<T> {
    vec![raw]
}

// ─── Collect into registry ──────────────────────────────────────────────────

/// Fold a batch of `Parsed<T>` results into a registry, deduplicating by ID.
fn collect_registry<T>(
    results: Vec<Parsed<T>>,
    get_id: impl Fn(&T) -> &str,
    kind_label: &str,
) -> (Registry<T>, Vec<LoadIssue>) {
    let mut registry = HashMap::new();
    let mut issues = Vec::new();

    for result in results {
        match result {
            Parsed::Err { issues: errs } => issues.extend(errs),
            Parsed::Ok {
                file,
                items,
                issues: validation_issues,
            } => {
                issues.extend(validation_issues);
                for item in items {
                    let id = get_id(&item).to_owned();
                    match registry.entry(id.clone()) {
                        Occupied(_) => {
                            issues.push(LoadIssue {
                                file: file.clone(),
                                message: format!("duplicate {kind_label} id '{id}'"),
                                path: None,
                            });
                        }
                        Vacant(entry) => {
                            entry.insert(Entry {
                                file: file.clone(),
                                item,
                            });
                        }
                    }
                }
            }
        }
    }

    (registry, issues)
}

// ─── Public entry point ─────────────────────────────────────────────────────

/// Load the full dataset from `data_dir`.
pub fn load_dataset(data_dir: &Path) -> Result<LoadedDataset, LoadError> {
    let base = data_dir.parent().unwrap_or(data_dir);
    let rel = |p: &Path| -> String {
        p.strip_prefix(base)
            .unwrap_or(p)
            .to_string_lossy()
            .into_owned()
    };

    let (questions, q_issues) = load_questions(data_dir, &rel)?;
    let (packs, p_issues) = load_packs(data_dir, &rel)?;
    let (tags, t_issues) = load_tags(data_dir, &rel)?;
    let (overlays, o_issues) = load_overlays(data_dir, &rel)?;
    let (games, g_issues) = load_games(data_dir, &rel)?;

    let issues = [q_issues, p_issues, t_issues, o_issues, g_issues].concat();

    Ok(LoadedDataset {
        data_dir: data_dir.to_string_lossy().into_owned(),
        questions,
        packs,
        tags,
        overlays,
        games,
        issues,
    })
}

// ─── Questions ──────────────────────────────────────────────────────────────

fn load_questions(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Question>, Vec<LoadIssue>), LoadError> {
    let files = walk_yaml(&data_dir.join("questions"))?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), parse_many, Some(garde_issues)))
        .collect();

    Ok(collect_registry(results, |q| q.id(), "question"))
}

// ─── Packs ──────────────────────────────────────────────────────────────────

fn load_packs(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Pack>, Vec<LoadIssue>), LoadError> {
    let files = walk_yaml(&data_dir.join("packs"))?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), parse_one, Some(garde_issues)))
        .collect();

    Ok(collect_registry(results, |p| &p.id, "pack"))
}

// ─── Tags ───────────────────────────────────────────────────────────────────

fn load_tags(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Tag>, Vec<LoadIssue>), LoadError> {
    let dir = data_dir.join("tags");

    let results: Vec<_> = TAG_CATEGORIES
        .iter()
        .filter_map(|cat| {
            let path = dir.join(format!("{cat}.yaml"));
            path.exists()
                .then(|| parse_file(&path, &rel(&path), parse_many, None))
        })
        .collect();

    Ok(collect_registry(results, |t| &t.id, "tag"))
}

// ─── Overlays ───────────────────────────────────────────────────────────────

fn load_overlays(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Overlays, Vec<LoadIssue>), LoadError> {
    let i18n_dir = data_dir.join("i18n");
    if !i18n_dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }

    let mut overlays: Overlays = HashMap::new();
    let mut issues = Vec::new();

    for locale_entry in read_dir_sorted(&i18n_dir)? {
        if !locale_entry.is_dir() {
            continue;
        }
        let locale = filename(&locale_entry);
        let locale_overlays = overlays.entry(locale).or_default();
        let (g_ovl, g_iss) = load_game_overlays(&locale_entry, rel)?;
        locale_overlays.games.extend(g_ovl);
        issues.extend(g_iss);

        let (q_ovl, q_iss) = load_question_overlays(&locale_entry, rel)?;
        let (p_ovl, p_iss) = load_pack_overlays(&locale_entry, rel)?;
        let (t_ovl, t_iss) = load_tag_overlays(&locale_entry, rel)?;

        locale_overlays.questions.extend(q_ovl);
        locale_overlays.packs.extend(p_ovl);
        locale_overlays.tags.extend(t_ovl);
        issues.extend([q_iss, p_iss, t_iss].concat());
    }

    Ok((overlays, issues))
}

fn load_question_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<QuestionOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("questions");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }
    let files = walk_yaml(&dir)?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), parse_many, None))
        .collect();

    Ok(collect_registry(results, |q| &q.id, "question overlay"))
}

fn load_pack_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<PackOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("packs");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }
    let files = walk_yaml(&dir)?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), parse_one, None))
        .collect();

    Ok(collect_registry(results, |p| &p.id, "pack overlay"))
}

fn load_tag_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<TagOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("tags");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }

    let results: Vec<_> = TAG_CATEGORIES
        .iter()
        .filter_map(|cat| {
            let path = dir.join(format!("{cat}.yaml"));
            path.exists()
                .then(|| parse_file(&path, &rel(&path), parse_many, None))
        })
        .collect();

    Ok(collect_registry(results, |t| &t.id, "tag overlay"))
}

// ─── Games ──────────────────────────────────────────────────────────────

fn load_games(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameConfig>, Vec<LoadIssue>), LoadError> {
    let files = walk_yaml(&data_dir.join("games"))?;
    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), parse_one, Some(game_config_issues)))
        .collect();
    Ok(collect_registry(
        results,
        |g: &GameConfig| &g.id,
        "game config",
    ))
}

fn load_game_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("games");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }
    let files = walk_yaml(&dir)?;
    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), parse_one, None))
        .collect();
    Ok(collect_registry(results, |g| &g.id, "game overlay"))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Recursively collect all `.yaml` / `.yml` files under `dir`.
fn walk_yaml(dir: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let mut files = Vec::new();
    if !dir.exists() {
        return Ok(files);
    }
    walk_yaml_inner(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_yaml_inner(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), LoadError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_yaml_inner(&path, out)?;
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy();
            if ext == "yaml" || ext == "yml" {
                out.push(path);
            }
        }
    }
    Ok(())
}

fn read_dir_sorted(dir: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    Ok(entries)
}

fn parse_yaml_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, serde_yaml::Error> {
    let text = fs::read_to_string(path)
        .map_err(|e| <serde_yaml::Error as serde::de::Error>::custom(format!("IO: {e}")))?;
    serde_yaml::from_str(&text)
}

fn yaml_issue(file: &str, e: serde_yaml::Error) -> LoadIssue {
    LoadIssue {
        file: file.to_owned(),
        message: e.to_string(),
        path: e
            .location()
            .map(|loc| format!("line {}, col {}", loc.line(), loc.column())),
    }
}

fn garde_issues<T: Validate<Context = ()>>(value: &T, file: &str) -> Vec<LoadIssue> {
    match value.validate() {
        Ok(()) => Vec::new(),
        Err(report) => report
            .iter()
            .map(|(path, error)| LoadIssue {
                file: file.to_owned(),
                message: error.message().to_string(),
                path: Some(path.to_string()),
            })
            .collect(),
    }
}

fn filename(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

// ponytail: custom validator for GameConfig — garde is overkill, just entry validation
fn game_config_issues(gc: &GameConfig, file: &str) -> Vec<LoadIssue> {
    let mut issues = Vec::new();

    if let Err(err) = valid_game_id(&gc.id, &()) {
        issues.push(LoadIssue {
            file: file.to_owned(),
            message: err.to_string(),
            path: None,
        });
    }

    if gc.games.is_empty() {
        issues.push(LoadIssue {
            file: file.to_owned(),
            message: "game config must define at least one game entry".to_owned(),
            path: None,
        });
    }

    issues.extend(gc.games.iter().enumerate().filter_map(|(i, entry)| {
        entry
            .validate()
            .map_err(|msg| LoadIssue {
                file: file.to_owned(),
                message: format!("game[{}]: {}", i, msg),
                path: None,
            })
            .err()
    }));

    issues
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
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
    fn catches_pack_without_content() {
        let ds = load(&with_registries(&[(
            "packs/empty.yaml",
            "id: pack_empty\ntitle: Empty\n",
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("at least one of"))
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
    fn catches_invalid_pack_id() {
        let ds = load(&with_registries(&[(
            "packs/bad.yaml",
            "id: notapackid\ntitle: Bad\nquestions: [q_alpha_one]\n",
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("invalid pack id"))
        );
    }

    #[test]
    fn catches_invalid_question_ref_in_pack() {
        let ds = load(&with_registries(&[(
            "packs/bad.yaml",
            "id: pack_bad\ntitle: Bad\nquestions: [NOT-VALID]\n",
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("invalid question id"))
        );
    }

    #[test]
    fn catches_invalid_includes_ref_in_pack() {
        let ds = load(&with_registries(&[(
            "packs/bad.yaml",
            "id: pack_bad\ntitle: Bad\nincludes: [not_a_pack_id]\n",
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("invalid pack id"))
        );
    }

    #[test]
    fn catches_invalid_tag_ref_in_pack_filter() {
        let ds = load(&with_registries(&[(
            "packs/bad.yaml",
            "id: pack_bad\ntitle: Bad\nfilter:\n  tags_any: [INVALID]\n",
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("invalid tag ref"))
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

    const VALID_GRID_GAME: &str = r#"
id: game_test_grid
title: Test Grid
description: A test grid game
games:
  - mode: grid_quiz
    title: Round 1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: this_question
      steal_policy: round_limited
      judge: auto
      question_timer_secs: 30
      answer_timer_secs: 15
    board:
      points: [100, 200, 300, 500]
      categories:
        - name: Capitals
          filter:
            tags_any: [subject:geo]
        - name: Flags
          question_ids: { 100: q_alpha_one, 200: q_alpha_two }
"#;

    const VALID_LINEAR_GAME: &str = r#"
id: game_test_linear
title: Test Linear
description: A test linear game
games:
  - mode: linear
    title: Round 1
    rules:
      buzz_policy: broadcast
      scoring_mode: all_grade
      lockout_policy: none
      steal_policy: none
      judge: auto
      question_timer_secs: 20
      answer_timer_secs: 10
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#;

    const VALID_QUESTION_TWO: &str = r#"
- id: q_alpha_two
  kind: text
  tags: [subject:history, difficulty:general]
  content:
    default_lang: en
    prompt: { text: "What is two plus two?" }
    answer: Four
    variants:
      open:
        accepted: ["Four", "4"]
"#;

    #[test]
    fn loads_valid_grid_game_config() {
        let ds = load(&with_registries(&[
            ("questions/test.yaml", VALID_QUESTION),
            ("questions/two.yaml", VALID_QUESTION_TWO),
            ("games/test.yaml", VALID_GRID_GAME),
        ]));
        assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
        assert_eq!(ds.games.len(), 1);
        let gc = ds.games.get("game_test_grid").unwrap();
        assert_eq!(gc.item.games.len(), 1);
        assert_eq!(gc.item.title, "Test Grid");
        match &gc.item.games[0] {
            GameEntry::GridQuiz(g) => {
                assert_eq!(g.title, "Round 1");
                assert!(matches!(g.rules.buzz_policy, BuzzPolicy::OpenFloor));
                assert!(matches!(g.rules.steal_policy, StealPolicy::RoundLimited));
                assert_eq!(g.board.points.len(), 4);
                assert_eq!(g.board.categories.len(), 2);
                assert_eq!(g.board.categories[1].name, "Flags");
            }
            _ => panic!("expected GridQuiz"),
        }
    }

    #[test]
    fn loads_valid_linear_game_config() {
        let ds = load(&with_registries(&[
            ("questions/test.yaml", VALID_QUESTION),
            ("games/test.yaml", VALID_LINEAR_GAME),
        ]));
        assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
        assert_eq!(ds.games.len(), 1);
        let gc = ds.games.get("game_test_linear").unwrap();
        match &gc.item.games[0] {
            GameEntry::Linear(g) => {
                assert!(matches!(g.rules.buzz_policy, BuzzPolicy::Broadcast));
                assert!(matches!(g.rules.scoring_mode, ScoringMode::AllGrade));
                assert!(matches!(g.rules.steal_policy, StealPolicy::None));
                match &g.questions {
                    LinearSource::Questions { question_ids } => {
                        assert_eq!(question_ids.as_slice(), &["q_alpha_one"]);
                    }
                    _ => panic!("expected Questions source"),
                }
            }
            _ => panic!("expected Linear"),
        }
    }

    #[test]
    fn loads_game_with_default_timers() {
        let ds = load(&with_registries(&[(
            "games/test.yaml",
            r#"
id: game_short
title: Short
description: Uses default timers
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
        )]));
        assert!(ds.issues.is_empty(), "unexpected issues: {:?}", ds.issues);
        let gc = ds.games.get("game_short").unwrap();
        match &gc.item.games[0] {
            GameEntry::GridQuiz(g) => {
                assert_eq!(g.rules.question_timer_secs, 30); // default
                assert_eq!(g.rules.answer_timer_secs, 15); // default
            }
            _ => panic!("expected GridQuiz"),
        }
    }

    #[test]
    fn catches_duplicate_game_ids() {
        let ds = load(&[
            (
                "games/a.yaml",
                r#"
id: game_dup
title: A
description: A
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
        - name: Math
          filter: { tags_any: [subject:math] }
        - name: Geo
          filter: { tags_any: [subject:geo] }
"#,
            ),
            (
                "games/b.yaml",
                r#"
id: game_dup
title: B
description: B
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
        - name: Math
          filter: { tags_any: [subject:math] }
        - name: Geo
          filter: { tags_any: [subject:geo] }
"#,
            ),
        ]);
        assert!(ds.issues.iter().any(|i| i.message.contains("duplicate")));
    }

    #[test]
    fn game_entry_validates_broadcast_first_correct() {
        let ds = load(&[(
            "games/bad.yaml",
            r#"
id: game_bad_combo
title: Bad
description: Bad combo
games:
  - mode: linear
    title: R1
    rules:
      buzz_policy: broadcast
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#,
        )]);
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("broadcast")
                    && i.message.to_lowercase().contains("first"))
        );
    }

    #[test]
    fn game_entry_validates_broadcast_steal() {
        let ds = load(&[(
            "games/bad.yaml",
            r#"
id: game_bad_steal
title: Bad
description: Bad
games:
  - mode: linear
    title: R1
    rules:
      buzz_policy: broadcast
      scoring_mode: all_grade
      lockout_policy: none
      steal_policy: open_floor
      judge: auto
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#,
        )]);
        assert!(ds.issues.iter().any(|i| i.message.contains("steal_policy")));
    }

    #[test]
    fn loads_board_with_explicit_question_ids() {
        let ds = load(&[("games/test.yaml", VALID_GRID_GAME)]);
        let gc = ds.games.get("game_test_grid").unwrap();
        match &gc.item.games[0] {
            GameEntry::GridQuiz(g) => {
                let flags = &g.board.categories[1];
                let ids = flags.question_ids.as_ref().unwrap();
                assert_eq!(ids.get(&100), Some(&"q_alpha_one".to_string()));
                assert_eq!(ids.get(&200), Some(&"q_alpha_two".to_string()));
            }
            _ => panic!("expected GridQuiz"),
        }
    }

    #[test]
    fn catches_game_with_single_category() {
        let ds = load(&[(
            "games/bad.yaml",
            r#"
id: game_onetop
title: Bad
description: Bad
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
        - name: Only One
          filter: { tags_any: [subject:math] }
"#,
        )]);
        assert!(!ds.issues.is_empty());
    }

    #[test]
    fn catches_invalid_game_id() {
        let ds = load(&[(
            "games/bad.yaml",
            r#"
id: not_a_game_id
title: Bad
description: Bad
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
        - name: Math
          filter: { tags_any: [subject:math] }
        - name: Geo
          filter: { tags_any: [subject:geo] }
"#,
        )]);
        assert!(ds.issues.iter().any(|i| i.message.contains("invalid game id")));
    }

    #[test]
    fn rejects_unknown_game_field() {
        let ds = load(&[(
            "games/bad.yaml",
            r#"
id: game_unknown_field
title: Bad
description: Bad
oops: true
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
        )]);
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("unknown field") && i.message.contains("oops"))
        );
    }

    #[test]
    fn catches_zero_timers_on_game_rules() {
        let ds = load(&with_registries(&[(
            "games/bad.yaml",
            r#"
id: game_zero_timer
title: Bad
description: Bad
games:
  - mode: linear
    title: R1
    rules:
      buzz_policy: open_floor
      scoring_mode: first_correct
      lockout_policy: none
      steal_policy: none
      judge: auto
      question_timer_secs: 0
      answer_timer_secs: 0
    questions:
      source: questions
      question_ids: [q_alpha_one]
"#,
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("question_timer_secs must be greater than 0"))
        );
    }

    #[test]
    fn catches_grid_points_not_strictly_increasing() {
        let ds = load(&with_registries(&[(
            "games/bad.yaml",
            r#"
id: game_bad_points
title: Bad
description: Bad
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
      points: [200, 100]
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
                .any(|i| i.message.contains("board points must be strictly increasing"))
        );
    }

    #[test]
    fn catches_grid_question_ids_key_not_in_points() {
        let ds = load(&with_registries(&[
            ("questions/test.yaml", VALID_QUESTION),
            (
                "games/bad.yaml",
                r#"
id: game_bad_keys
title: Bad
description: Bad
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
          question_ids: { 300: q_alpha_one }
        - name: History
          filter: { tags_any: [subject:history] }
"#,
            ),
        ]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("question_ids key 300 must be present in board.points"))
        );
    }

    #[test]
    fn catches_grid_duplicate_explicit_question_ids() {
        let ds = load(&with_registries(&[
            ("questions/test.yaml", VALID_QUESTION),
            (
                "games/bad.yaml",
                r#"
id: game_dup_qids
title: Bad
description: Bad
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
          question_ids: { 100: q_alpha_one }
        - name: History
          question_ids: { 200: q_alpha_one }
"#,
            ),
        ]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("explicit question id 'q_alpha_one' is duplicated"))
        );
    }

    #[test]
    fn catches_linear_duplicate_question_ids() {
        let ds = load(&with_registries(&[(
            "games/bad.yaml",
            r#"
id: game_dup_linear
title: Bad
description: Bad
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
      question_ids: [q_alpha_one, q_alpha_one]
"#,
        )]));
        assert!(
            ds.issues
                .iter()
                .any(|i| i.message.contains("has duplicate question id 'q_alpha_one'"))
        );
    }

    #[test]
    fn catches_difficulty_map_key_not_in_points() {
        let ds = load(&with_registries(&[(
            "games/bad.yaml",
            r#"
id: game_bad_diff_key
title: Bad
description: Bad
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
        300: [subject:geo]
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
                .any(|i| i.message.contains("difficulty_map key 300 must be present in board.points"))
        );
    }

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

    #[test]
    fn loads_real_dataset_without_issues() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data");
        if !path.exists() {
            return;
        }
        let mut ds = load_dataset(&path).expect("load_dataset failed");
        let cross = crate::data::validate::run_cross_file_checks(&ds);
        ds.issues.extend(cross);
        assert!(ds.issues.is_empty(), "real dataset issues: {:?}", ds.issues);
        assert!(ds.questions.len() >= 20);
        assert!(ds.packs.len() >= 1);
        assert!(ds.tags.len() >= 10);
    }
}
