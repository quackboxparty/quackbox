//! Domain types for the data layer.

mod common;
mod game;
mod media;
mod overlay;
mod pack;
mod question;
mod tag;

pub use common::*;
pub use game::*;
pub use media::*;
pub use overlay::*;
pub use pack::*;
pub use question::*;
pub use tag::*;

use std::collections::HashMap;

/// One item with its source file path.
#[derive(Debug, Clone)]
pub struct Entry<T> {
    pub file: String,
    pub item: T,
}

/// Registry of items keyed by their string ID.
pub type Registry<T> = HashMap<String, Entry<T>>;

/// Per-locale translation overlays.
#[derive(Debug, Clone, Default)]
pub struct LocaleOverlays {
    pub questions: Registry<QuestionOverlay>,
    pub packs: Registry<PackOverlay>,
    pub tags: Registry<TagOverlay>,
    pub games: Registry<GameOverlay>,
}

pub type Overlays = HashMap<String, LocaleOverlays>;

/// The full loaded dataset, ready for cross-file checks and querying.
#[derive(Debug, Clone)]
pub struct Dataset {
    pub data_dir: String,
    pub questions: Registry<Question>,
    pub packs: Registry<Pack>,
    pub tags: Registry<Tag>,
    pub overlays: Overlays,
    pub games: Registry<GameConfig>,
    pub issues: Vec<LoadIssue>,
}

/// Non-fatal diagnostic from the data loader.
#[derive(Debug, Clone)]
pub struct LoadIssue {
    pub file: String,
    pub message: String,
    pub path: Option<String>,
}

impl std::fmt::Display for LoadIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path {
            Some(p) => write!(f, "{} at {}: {}", self.file, p, self.message),
            None => write!(f, "{}: {}", self.file, self.message),
        }
    }
}
