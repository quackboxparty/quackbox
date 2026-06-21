use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
pub struct Source {
    pub url: String,
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

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
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
