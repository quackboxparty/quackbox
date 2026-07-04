use serde::Deserialize;

use super::media::Media;

/// Translatable subset of a question — no `correct`, `position`, numeric
/// `answer`, variant config, or metadata fields.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestionOverlay {
    pub id: String,
    pub content: ContentOverlay,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentOverlay {
    #[serde(default)]
    pub answer: Option<String>,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub prompt: Option<PromptOverlay>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<OrderItemOverlay>>,
    #[serde(default)]
    pub variants: Option<VariantsOverlay>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptOverlay {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub media: Option<Vec<Media>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantsOverlay {
    #[serde(default)]
    pub multiple_choice: Option<MultipleChoiceOverlay>,
    #[serde(default)]
    pub open: Option<OpenVariantOverlay>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultipleChoiceOverlay {
    pub choices: Vec<ChoiceOverlay>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceOverlay {
    pub id: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub media: Option<Vec<Media>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenVariantOverlay {
    #[serde(default)]
    pub accepted: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderItemOverlay {
    pub id: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub media: Option<Vec<Media>>,
}
