//! Fatal errors that stop the data load.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse: {path}: {source}")]
    Yaml {
        path: String,
        source: serde_yaml::Error,
    },
}
