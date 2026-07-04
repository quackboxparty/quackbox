use garde::Validate;
use serde::{Deserialize, Serialize};
/// Question kinds.

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum License {
    #[serde(rename = "CC0-1.0")]
    CC0_1_0,
    #[serde(rename = "CC-BY-4.0")]
    CC_BY_4_0,
    #[serde(rename = "CC-BY-SA-4.0")]
    CC_BY_SA_4_0,
    #[serde(rename = "CC-BY-NC-4.0")]
    CC_BY_NC_4_0,
    #[serde(rename = "CC-BY-ND-4.0")]
    CC_BY_ND_4_0,
    #[serde(rename = "MIT")]
    MIT,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct Source {
    #[garde(pattern(r"^https?://"))]
    pub url: String,
    #[garde(pattern(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$"))]
    #[serde(default)]
    pub accessed: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deprecation {
    pub reason: String,
    #[serde(default)]
    pub replaced_by: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum QuestionKind {
    Text,
    Numeric,
    Order,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum VariantName {
    MultipleChoice,
    TrueFalse,
    Open,
    NumericInput,
    Range,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NormalizeOp {
    Lowercase,
    StripDiacritics,
    StripPunctuation,
    StripWhitespace,
    StripArticles,
}

pub const TAG_CATEGORIES: &[&str] = &[
    "subject",
    "difficulty",
    "audience",
    "region",
    "format",
    "warning",
];

use regex::Regex;
use std::sync::LazyLock;

static TAG_REF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(subject|difficulty|audience|region|format|warning):[a-z0-9][a-z0-9_]*$").unwrap()
});
static SLUG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9_]*$").unwrap());
static QUESTION_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^q_[a-z0-9][a-z0-9_]*$").unwrap());
static PACK_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^pack_[a-z0-9][a-z0-9_]*$").unwrap());
static GAME_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^game_[a-z0-9][a-z0-9_]*$").unwrap());
static LOCALE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z]{2}(-[A-Z]{2})?$").unwrap());

fn valid_by_regex(value: &str, re: &Regex, kind: &str) -> garde::Result {
    if re.is_match(value) {
        Ok(())
    } else {
        Err(garde::Error::new(format!("invalid {kind}: '{value}'")))
    }
}

pub fn valid_tag_refs(value: &[String], _ctx: &()) -> garde::Result {
    for v in value {
        valid_by_regex(v, &TAG_REF_RE, "tag ref")?;
    }
    Ok(())
}

pub fn valid_opt_tag_refs(value: &Option<Vec<String>>, _ctx: &()) -> garde::Result {
    for v in value.iter().flatten() {
        valid_by_regex(v, &TAG_REF_RE, "tag ref")?;
    }
    Ok(())
}

pub fn valid_slug(value: &str, _ctx: &()) -> garde::Result {
    valid_by_regex(value, &SLUG_RE, "slug")
}

pub fn valid_question_id(value: &str, _ctx: &()) -> garde::Result {
    valid_by_regex(value, &QUESTION_ID_RE, "question id")
}

pub fn valid_opt_question_ids(value: &Option<Vec<String>>, _ctx: &()) -> garde::Result {
    for v in value.iter().flatten() {
        valid_by_regex(v, &QUESTION_ID_RE, "question id")?;
    }
    Ok(())
}

pub fn valid_pack_id(value: &str, _ctx: &()) -> garde::Result {
    valid_by_regex(value, &PACK_ID_RE, "pack id")
}

pub fn valid_pack_ids(value: &Option<Vec<String>>, _ctx: &()) -> garde::Result {
    for v in value.iter().flatten() {
        valid_by_regex(v, &PACK_ID_RE, "pack id")?;
    }
    Ok(())
}

pub fn valid_game_id(value: &str, _ctx: &()) -> garde::Result {
    valid_by_regex(value, &GAME_ID_RE, "game id")
}

pub fn valid_locale(value: &str, _ctx: &()) -> garde::Result {
    valid_by_regex(value, &LOCALE_RE, "locale")
}

pub fn valid_opt_locale(value: &Option<String>, _ctx: &()) -> garde::Result {
    match value {
        Some(v) => valid_locale(v, _ctx),
        None => Ok(()),
    }
}
