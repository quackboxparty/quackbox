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
    // local: — relative path, no leading / or ..
    if let Some(sub) = value.strip_prefix("local:") {
        if sub.is_empty() || sub.starts_with('/') || sub.contains("..") {
            return Err(garde::Error::new(
                "local: ref must be a relative path without leading / or ..",
            ));
        }
        return Ok(());
    }
    // url:https://...
    if value.starts_with("url:https://") {
        return Ok(());
    }
    // youtube:<id>[?query]
    if let Some(rest) = value.strip_prefix("youtube:") {
        let id_part = rest.split('?').next().unwrap_or("");
        if (8..=24).contains(&id_part.len())
            && id_part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Ok(());
        }
        return Err(garde::Error::new(
            "youtube: ref must have an 8-24 char alphanumeric ID",
        ));
    }
    Err(garde::Error::new(
        "media ref must start with local:, url:https://, or youtube:",
    ))
}
