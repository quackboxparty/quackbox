//! Shared test utilities for data layer tests.

use std::fs;
use tempfile::TempDir;

use super::loader::load_dataset;
use super::types::Dataset;
use super::validate::run_cross_file_checks;

/// Create a temp data dir, write files, load + cross-validate, return dataset.
pub fn load(files: &[(&str, &str)]) -> Dataset {
    let tmp = fixture(files);
    let mut ds = load_dataset(tmp.path()).expect("load_dataset failed");
    let cross = run_cross_file_checks(&ds);
    ds.issues.extend(cross);
    ds
}

/// Create a temp dir with the standard subdirs and write the given files.
pub fn fixture(files: &[(&str, &str)]) -> TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    for dir in &["questions", "packs", "tags", "i18n", "media"] {
        fs::create_dir_all(tmp.path().join(dir)).unwrap();
    }
    for (path, content) in files {
        let full = tmp.path().join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, content).unwrap();
    }
    tmp
}

pub const VALID_QUESTION: &str = r#"
- id: q_alpha_one
  kind: text
  tags: [subject:geo, difficulty:general]
  content:
    default_lang: en
    prompt: { text: "What is one plus one?" }
    answer: Two
    variants:
      multiple_choice:
        choices:
          - { id: two, text: Two, correct: true }
          - { id: three, text: Three }
      open:
        accepted: ["Two", "2"]
"#;

pub const VALID_PACK: &str = r#"id: pack_alpha
title: Alpha Pack
questions: [q_alpha_one]
"#;

pub fn valid_registries() -> Vec<(&'static str, &'static str)> {
    vec![
        ("tags/audience.yaml", "[]\n"),
        (
            "tags/difficulty.yaml",
            "- id: difficulty:general\n  default_lang: en\n  label: General\n",
        ),
        ("tags/format.yaml", "[]\n"),
        ("tags/region.yaml", "[]\n"),
        (
            "tags/subject.yaml",
            "- id: subject:geo\n  default_lang: en\n  label: Geography\n\
             - id: subject:history\n  default_lang: en\n  label: History\n",
        ),
        ("tags/warning.yaml", "[]\n"),
    ]
}

pub fn with_registries<'a>(extra: &'a [(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
    let mut files = valid_registries();
    files.extend_from_slice(extra);
    files
}
