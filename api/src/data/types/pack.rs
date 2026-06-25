use garde::Validate;
use serde::Deserialize;

use super::common::*;

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct PackFilter {
    #[serde(default)]
    pub kinds: Option<Vec<QuestionKind>>,
    #[garde(custom(valid_opt_tag_refs))]
    #[serde(default)]
    pub tags_all: Option<Vec<String>>,
    #[garde(custom(valid_opt_tag_refs))]
    #[serde(default)]
    pub tags_any: Option<Vec<String>>,
    #[garde(custom(valid_opt_tag_refs))]
    #[serde(default)]
    pub tags_none: Option<Vec<String>>,
    #[serde(default)]
    pub variants_any: Option<Vec<VariantName>>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[garde(custom(pack_has_content))]
pub struct Pack {
    #[garde(custom(valid_pack_id))]
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<License>,
    #[garde(custom(valid_opt_locale))]
    #[serde(default)]
    pub default_lang: Option<String>,
    #[serde(default)]
    pub recommended_gamemodes: Option<Vec<String>>,
    #[garde(custom(valid_pack_ids))]
    #[serde(default)]
    pub includes: Option<Vec<String>>,
    #[garde(custom(valid_opt_question_ids))]
    #[serde(default)]
    pub questions: Option<Vec<String>>,
    #[garde(dive)]
    #[serde(default)]
    pub filter: Option<PackFilter>,
}

fn pack_has_content(pack: &Pack, _ctx: &()) -> garde::Result {
    let has_includes = pack.includes.as_ref().is_some_and(|v| !v.is_empty());
    let has_questions = pack.questions.as_ref().is_some_and(|v| !v.is_empty());
    let has_filter = pack.filter.is_some();
    if has_includes || has_questions || has_filter {
        Ok(())
    } else {
        Err(garde::Error::new(
            "pack must define at least one of: includes, questions, filter",
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct PackOverlay {
    #[garde(custom(valid_pack_id))]
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}
