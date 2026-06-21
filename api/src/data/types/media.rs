use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Audio,
    Video,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[garde(allow_unvalidated)]
pub struct Media {
    pub kind: MediaKind,
    #[garde(custom(valid_media_ref))]
    #[serde(rename = "ref")]
    pub media_ref: String,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u32>,
    #[serde(default)]
    pub start_ms: Option<u32>,
    #[serde(default)]
    pub end_ms: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

fn valid_media_ref(value: &str, _ctx: &()) -> garde::Result {
    if value.starts_with("local:")
        || value.starts_with("url:https://")
        || value.starts_with("youtube:")
    {
        Ok(())
    } else {
        Err(garde::Error::new(
            "media ref must start with local:, url:https://, or youtube:",
        ))
    }
}
