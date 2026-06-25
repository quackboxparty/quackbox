//! Fatal errors that stop the data load.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}
