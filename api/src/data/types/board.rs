use garde::Validate;
use serde::Deserialize;
use std::collections::HashMap;

use super::common::*;
use super::pack::PackFilter;

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct BoardFile {
    #[garde(custom(valid_board_id))]
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[garde(length(min = 2))]
    pub points: Vec<u32>,
    #[serde(default)]
    pub difficulty_map: Option<HashMap<String, Vec<String>>>,
    #[garde(length(min = 2), dive)]
    pub categories: Vec<BoardCategory>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[garde(custom(category_has_source))]
pub struct BoardCategory {
    pub name: String,
    #[serde(default)]
    pub question_ids: Option<HashMap<String, String>>,
    #[serde(default)]
    pub pack_ref: Option<String>,
    #[garde(dive)]
    #[serde(default)]
    pub filter: Option<PackFilter>,
}

fn category_has_source(cat: &BoardCategory, _ctx: &()) -> garde::Result {
    if cat.question_ids.is_some() || cat.pack_ref.is_some() || cat.filter.is_some() {
        Ok(())
    } else {
        Err(garde::Error::new(
            "category must define at least one of: question_ids, pack_ref, filter",
        ))
    }
}
