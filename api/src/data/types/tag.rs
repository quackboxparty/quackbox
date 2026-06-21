use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    pub id: String,
    pub label: String,
    pub default_lang: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TagOverlay {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}
