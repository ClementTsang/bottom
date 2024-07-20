use std::result;

use thiserror::Error;

/// A type alias for handling errors related to Bottom.
pub type Result<T> = result::Result<T, BottomError>;

/// An error that can occur while Bottom runs.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BottomError {
    /// An error when there is an IO exception.
    #[error("IO exception, {0}")]
    InvalidIo(String),
    /// An error to represent generic errors.
    #[error("Error, {0}")]
    GenericError(String),
}

impl From<std::io::Error> for BottomError {
    fn from(err: std::io::Error) -> Self {
        BottomError::InvalidIo(err.to_string())
    }
}
