//! Error code related to drawing.

use thiserror::Error;

/// A type alias for handling drawing-related errors.
pub type DrawResult<T> = std::result::Result<T, DrawError>;

/// The errors that can happen with drawing.
#[derive(Debug, Error)]
pub enum DrawError {
    /// An error when there is an IO exception.
    #[error(transparent)]
    InvalidIo(#[from] std::io::Error),
}
