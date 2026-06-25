//! Data layer — load, validate, and query the quiz content pool.
//!
//! Entry point: `data::load("data")` at server startup.

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

/// Load all YAML data from `data_dir` and validate cross-file refs.
/// Returns the ready-to-query dataset; non-fatal problems are in `issues`.
pub fn load(data_dir: &str) -> Result<LoadedDataset, LoadError> {
    let mut dataset = load_dataset(Path::new(data_dir))?;
    dataset.issues.extend(run_cross_file_checks(&dataset));
    Ok(dataset)
}
