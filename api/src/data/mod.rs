//! Data layer — load, validate, and query the quiz content pool.
//!
//! Entry point: `DataStore::load("data")` at server startup.

mod board;
mod error;
mod loader;
mod query;
mod types;
mod validate;

#[cfg(test)]
mod test_helpers;

pub use board::{build_board, BoardGrid, BuildBoardOpts};
pub use error::LoadError;
pub use loader::load_dataset;
pub use query::query_pool;
pub use types::*;
pub use validate::run_cross_file_checks;

use std::path::Path;

/// Top-level handle exposed to the rest of the app.
/// Owns the loaded dataset and provides query methods.
pub struct DataStore {
    pub dataset: LoadedDataset,
}

impl DataStore {
    /// Load all YAML data from `data_dir`, validate cross-file refs,
    /// and return a ready-to-query store.
    pub fn load(data_dir: &str) -> Result<Self, LoadError> {
        let path = Path::new(data_dir);
        let mut dataset = load_dataset(path)?;

        let cross_issues = run_cross_file_checks(&dataset);
        dataset.issues.extend(cross_issues);

        Ok(Self { dataset })
    }

    pub fn questions(&self) -> &Registry<Question> {
        &self.dataset.questions
    }

    pub fn packs(&self) -> &Registry<Pack> {
        &self.dataset.packs
    }

    pub fn tags(&self) -> &Registry<Tag> {
        &self.dataset.tags
    }

    pub fn issues(&self) -> &[LoadIssue] {
        &self.dataset.issues
    }
}
