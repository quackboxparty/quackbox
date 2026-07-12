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

impl MultipleChoiceVariant {
    /// IDs of choices marked `correct: true`. Empty if none.
    pub fn correct_choice_ids(&self) -> Vec<String> {
        self.choices
            .iter()
            .filter(|c| c.correct == Some(true))
            .map(|c| c.id.clone())
            .collect()
    }
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
pub struct TrueFalseVariant {
    pub correct: bool,
}

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
    #[garde(range(min = 0.0))]
    #[serde(default)]
    pub tolerance: f64,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Correctness {
    MultipleChoice {
        correct_ids: Vec<String>,
    },
    Open {
        accepted: Vec<String>,
    },
    TrueFalse {
        correct: bool,
    },
    /// NumericInput and Range share a correctness shape
    Numeric {
        value: f64,
        tolerance: f64,
    },
    Order {
        positions: Vec<(String, u32)>,
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

    /// Prompt shared by every variant — TextContent / NumericContent /
    /// OrderContent all embed the same `Prompt` type, so this is the one
    /// accessor projection code reaches for.
    pub fn prompt(&self) -> &Prompt {
        match self {
            Self::Text { content, .. } => &content.prompt,
            Self::Numeric { content, .. } => &content.prompt,
            Self::Order { content, .. } => &content.prompt,
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

    pub fn explanation(&self) -> Option<&str> {
        match self {
            Self::Text { content, .. } => content.explanation.as_deref(),
            Self::Numeric { content, .. } => content.explanation.as_deref(),
            Self::Order { content, .. } => content.explanation.as_deref(),
        }
    }

    pub fn default_lang(&self) -> &str {
        match self {
            Self::Text { content, .. } => &content.default_lang,
            Self::Numeric { content, .. } => &content.default_lang,
            Self::Order { content, .. } => &content.default_lang,
        }
    }

    pub fn mc_choices(&self) -> Option<&MultipleChoiceVariant> {
        match self {
            Self::Text { content, .. } => content.variants.multiple_choice.as_ref(),
            Self::Numeric { content, .. } => content.variants.multiple_choice.as_ref(),
            Self::Order { .. } => None,
        }
    }

    pub fn range(&self) -> Option<&RangeVariant> {
        match self {
            Self::Numeric { content, .. } => content.variants.range.as_ref(),
            _ => None,
        }
    }

    pub fn order_items(&self) -> Option<&[OrderItem]> {
        match self {
            Self::Order { content, .. } => Some(&content.items),
            _ => None,
        }
    }

    /// Correctness for the resolved variant. `None` when the variant isn't
    /// defined on this kind. Order ignores `variant` — its shape is fixed.
    pub fn correctness(&self, variant: VariantName) -> Option<Correctness> {
        match self {
            Self::Text { content, .. } => match variant {
                VariantName::MultipleChoice => content
                    .variants
                    .multiple_choice
                    .as_ref()
                    .map(|mc| Correctness::MultipleChoice {
                        correct_ids: mc.correct_choice_ids(),
                    }),
                VariantName::Open => content
                    .variants
                    .open
                    .as_ref()
                    .map(|o| Correctness::Open {
                        accepted: o.accepted.clone(),
                    }),
                VariantName::TrueFalse => content
                    .variants
                    .true_false
                    .as_ref()
                    .map(|tf| Correctness::TrueFalse {
                        correct: tf.correct,
                    }),
                _ => None,
            },
            Self::Numeric { content, .. } => match variant {
                VariantName::MultipleChoice => content
                    .variants
                    .multiple_choice
                    .as_ref()
                    .map(|mc| Correctness::MultipleChoice {
                        correct_ids: mc.correct_choice_ids(),
                    }),
                VariantName::NumericInput => content
                    .variants
                    .numeric_input
                    .as_ref()
                    .map(|ni| Correctness::Numeric {
                        value: content.answer,
                        tolerance: ni.tolerance,
                    }),
                VariantName::Range => content.variants.range.as_ref().map(|r| Correctness::Numeric {
                    value: content.answer,
                    tolerance: r.tolerance,
                }),
                _ => None,
            },
            Self::Order { content, .. } => Some(Correctness::Order {
                positions: content
                    .items
                    .iter()
                    .map(|i| (i.id.clone(), i.position))
                    .collect(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_q(s: &str) -> Question {
        serde_yaml::from_str(s).expect("test fixture must parse")
    }

    const TF_Q: &str = r#"
kind: text
id: q_tf
tags: []
content:
  default_lang: en
  prompt: { text: "The sky is blue." }
  answer: "true"
  variants:
    true_false: { correct: true }
"#;

    const NUM_Q: &str = r#"
kind: numeric
id: q_n
tags: []
content:
  default_lang: en
  prompt: { text: "Answer" }
  answer: 42
  variants:
    numeric_input: { tolerance: 0.5 }
    range: { min: 0, max: 100, step: 1, tolerance: 2 }
"#;

    const ORDER_Q: &str = r#"
kind: order
id: q_o
tags: []
content:
  default_lang: de
  prompt: { text: "Order these." }
  items:
    - { id: a, text: A, position: 1 }
    - { id: b, text: B, position: 2 }
"#;

    #[test]
    fn true_false_correctness_reads_correct_field() {
        // Regression for the empty-{} bug: TrueFalseVariant now carries correct.
        let q = load_q(TF_Q);
        assert_eq!(
            q.correctness(VariantName::TrueFalse),
            Some(Correctness::TrueFalse { correct: true })
        );
    }

    #[test]
    fn correctness_none_for_unsupported_variant() {
        let q = load_q(TF_Q);
        assert_eq!(q.correctness(VariantName::NumericInput), None);
    }

    #[test]
    fn numeric_input_correctness_carries_tolerance() {
        let q = load_q(NUM_Q);
        assert_eq!(
            q.correctness(VariantName::NumericInput),
            Some(Correctness::Numeric { value: 42.0, tolerance: 0.5 })
        );
    }

    #[test]
    fn range_correctness_uses_own_tolerance() {
        let q = load_q(NUM_Q);
        assert_eq!(
            q.correctness(VariantName::Range),
            Some(Correctness::Numeric { value: 42.0, tolerance: 2.0 })
        );
    }

    #[test]
    fn order_correctness_ignores_variant_arg() {
        let q = load_q(ORDER_Q);
        assert_eq!(
            q.correctness(VariantName::MultipleChoice),
            Some(Correctness::Order {
                positions: vec![("a".into(), 1), ("b".into(), 2)]
            })
        );
    }

    #[test]
    fn accessors_return_expected_per_kind() {
        let tf = load_q(TF_Q);
        assert_eq!(tf.default_lang(), "en");
        assert!(tf.explanation().is_none());
        assert!(tf.mc_choices().is_none());
        assert!(tf.range().is_none());
        assert!(tf.order_items().is_none());

        let num = load_q(NUM_Q);
        assert!(num.range().is_some());
        assert!(num.mc_choices().is_none());
        assert!(num.order_items().is_none());

        let order = load_q(ORDER_Q);
        assert_eq!(order.default_lang(), "de");
        assert!(order.order_items().is_some());
        assert!(order.mc_choices().is_none());
        assert!(order.range().is_none());
    }
}
