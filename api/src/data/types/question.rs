use garde::Validate;
use serde::Deserialize;
use std::collections::HashSet;

use super::common::*;
use super::media::{Media, MediaKind};

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct Prompt {
    pub text: String,
    #[garde(dive)]
    #[serde(default)]
    pub media: Option<Vec<Media>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct Choice {
    #[garde(custom(valid_slug))]
    pub id: String,
    pub text: String,
    pub correct: Option<bool>,
    #[garde(dive)]
    #[serde(default)]
    pub media: Option<Vec<Media>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct MultipleChoiceVariant {
    #[garde(
        length(min = 2),
        custom(choices_have_correct),
        custom(choices_unique_ids),
        dive
    )]
    pub choices: Vec<Choice>,
}

fn choices_have_correct(choices: &[Choice], _ctx: &()) -> garde::Result {
    if choices.iter().any(|c| c.correct == Some(true)) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "multiple_choice requires at least one choice with correct: true",
        ))
    }
}

fn choices_unique_ids(choices: &[Choice], _ctx: &()) -> garde::Result {
    let mut seen = HashSet::new();
    for c in choices {
        if !seen.insert(&c.id) {
            return Err(garde::Error::new(format!("duplicate choice id: {}", c.id)));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct OpenVariant {
    #[garde(length(min = 1))]
    pub accepted: Vec<String>,
    #[serde(default)]
    pub normalize: Option<Vec<NormalizeOp>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct TrueFalseVariant {}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct NumericInputVariant {
    #[garde(range(min = 0.0))]
    #[serde(default)]
    pub tolerance: f64,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
#[garde(custom(range_max_gt_min))]
pub struct RangeVariant {
    pub min: f64,
    pub max: f64,
    #[garde(range(min = 0.0))]
    #[serde(default = "default_step")]
    pub step: f64,
}

fn range_max_gt_min(value: &RangeVariant, _ctx: &()) -> garde::Result {
    if value.max > value.min {
        Ok(())
    } else {
        Err(garde::Error::new(
            "range.max must be greater than range.min",
        ))
    }
}

fn default_step() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(custom(text_has_variant))]
pub struct TextVariants {
    #[serde(default)]
    #[garde(dive)]
    pub multiple_choice: Option<MultipleChoiceVariant>,
    #[serde(default)]
    #[garde(dive)]
    pub open: Option<OpenVariant>,
    #[serde(default)]
    #[garde(dive)]
    pub true_false: Option<TrueFalseVariant>,
}

fn text_has_variant(v: &TextVariants, _ctx: &()) -> garde::Result {
    if v.multiple_choice.is_some() || v.open.is_some() || v.true_false.is_some() {
        Ok(())
    } else {
        Err(garde::Error::new(
            "text question must define at least one variant",
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(custom(numeric_has_variant))]
pub struct NumericVariants {
    #[serde(default)]
    #[garde(dive)]
    pub multiple_choice: Option<MultipleChoiceVariant>,
    #[serde(default)]
    #[garde(dive)]
    pub numeric_input: Option<NumericInputVariant>,
    #[serde(default)]
    #[garde(dive)]
    pub range: Option<RangeVariant>,
}

fn numeric_has_variant(v: &NumericVariants, _ctx: &()) -> garde::Result {
    if v.multiple_choice.is_some() || v.numeric_input.is_some() || v.range.is_some() {
        Ok(())
    } else {
        Err(garde::Error::new(
            "numeric question must define at least one variant",
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct TextContent {
    #[garde(custom(valid_locale))]
    pub default_lang: String,
    #[garde(dive)]
    pub prompt: Prompt,
    pub answer: String,
    pub explanation: Option<String>,
    #[garde(dive)]
    pub variants: TextVariants,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct NumericContent {
    #[garde(custom(valid_locale))]
    pub default_lang: String,
    #[garde(dive)]
    pub prompt: Prompt,
    pub answer: f64,
    pub unit: Option<String>,
    pub explanation: Option<String>,
    #[garde(dive)]
    pub variants: NumericVariants,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct OrderItem {
    #[garde(custom(valid_slug))]
    pub id: String,
    pub text: String,
    #[garde(range(min = 1))]
    pub position: u32,
    #[garde(dive)]
    #[serde(default)]
    pub media: Option<Vec<Media>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct OrderContent {
    #[garde(custom(valid_locale))]
    pub default_lang: String,
    #[garde(dive)]
    pub prompt: Prompt,
    #[garde(length(min = 2), custom(order_items_valid), dive)]
    pub items: Vec<OrderItem>,
    pub explanation: Option<String>,
}

fn order_items_valid(items: &[OrderItem], _ctx: &()) -> garde::Result {
    // Unique IDs
    let mut ids = HashSet::new();
    for it in items {
        if !ids.insert(&it.id) {
            return Err(garde::Error::new(format!(
                "duplicate order item id: {}",
                it.id
            )));
        }
    }
    // Contiguous positions 1..N
    let mut positions: Vec<u32> = items.iter().map(|i| i.position).collect();
    positions.sort();
    for (idx, &pos) in positions.iter().enumerate() {
        if pos != (idx as u32) + 1 {
            return Err(garde::Error::new(
                "order items must have contiguous positions starting at 1",
            ));
        }
    }
    Ok(())
}

/// Base metadata shared by all question kinds.
#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct QuestionBase {
    #[garde(custom(valid_question_id))]
    pub id: String,
    #[garde(custom(valid_tag_refs))]
    pub tags: Vec<String>,
    pub deprecated: Option<Deprecation>,
    #[garde(custom(valid_opt_locale))]
    pub lang_locked: Option<String>,
    pub license: Option<License>,
    #[garde(dive)]
    pub sources: Option<Vec<Source>>,
}

/// Discriminated union over `kind: text | numeric | order`.
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Question {
    Text {
        #[serde(flatten)]
        #[garde(dive)]
        base: QuestionBase,
        #[garde(dive)]
        content: TextContent,
    },
    Numeric {
        #[serde(flatten)]
        #[garde(dive)]
        base: QuestionBase,
        #[garde(dive)]
        content: NumericContent,
    },
    Order {
        #[serde(flatten)]
        #[garde(dive)]
        base: QuestionBase,
        #[garde(dive)]
        content: OrderContent,
    },
}

impl Question {
    pub fn id(&self) -> &str {
        self.base().id.as_str()
    }

    pub fn tags(&self) -> &[String] {
        &self.base().tags
    }

    pub fn kind(&self) -> QuestionKind {
        match self {
            Self::Text { .. } => QuestionKind::Text,
            Self::Numeric { .. } => QuestionKind::Numeric,
            Self::Order { .. } => QuestionKind::Order,
        }
    }

    pub fn base(&self) -> &QuestionBase {
        match self {
            Self::Text { base, .. } | Self::Numeric { base, .. } | Self::Order { base, .. } => base,
        }
    }

    /// Variant names defined on this question (`order` has none).
    pub fn variant_names(&self) -> HashSet<VariantName> {
        let mut set = HashSet::new();
        match self {
            Self::Text { content, .. } => {
                if content.variants.multiple_choice.is_some() {
                    set.insert(VariantName::MultipleChoice);
                }
                if content.variants.open.is_some() {
                    set.insert(VariantName::Open);
                }
                if content.variants.true_false.is_some() {
                    set.insert(VariantName::TrueFalse);
                }
            }
            Self::Numeric { content, .. } => {
                if content.variants.multiple_choice.is_some() {
                    set.insert(VariantName::MultipleChoice);
                }
                if content.variants.numeric_input.is_some() {
                    set.insert(VariantName::NumericInput);
                }
                if content.variants.range.is_some() {
                    set.insert(VariantName::Range);
                }
            }
            Self::Order { .. } => {}
        }
        set
    }

    /// Collect all media refs from prompt + choices/items.
    pub fn media_refs(&self) -> Vec<(&str, MediaKind)> {
        let mut out = Vec::new();

        fn push_media<'a>(out: &mut Vec<(&'a str, MediaKind)>, media: &'a Option<Vec<Media>>) {
            if let Some(items) = media {
                for m in items {
                    out.push((m.media_ref.as_str(), m.kind));
                }
            }
        }

        match self {
            Self::Text { content, .. } => {
                push_media(&mut out, &content.prompt.media);
                if let Some(mc) = &content.variants.multiple_choice {
                    for c in &mc.choices {
                        push_media(&mut out, &c.media);
                    }
                }
            }
            Self::Numeric { content, .. } => {
                push_media(&mut out, &content.prompt.media);
                if let Some(mc) = &content.variants.multiple_choice {
                    for c in &mc.choices {
                        push_media(&mut out, &c.media);
                    }
                }
            }
            Self::Order { content, .. } => {
                push_media(&mut out, &content.prompt.media);
                for item in &content.items {
                    push_media(&mut out, &item.media);
                }
            }
        }
        out
    }
}
