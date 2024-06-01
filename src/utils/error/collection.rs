//! Error code related to data collection.

use std::{borrow::Cow, num::ParseFloatError, str::Utf8Error};

use thiserror::Error;

/// A type alias for handling collection-related errors.
pub type CollectionResult<T> = std::result::Result<T, CollectionError>;

/// The errors that can happen with data collection.
#[derive(Debug, Error)]
pub enum CollectionError {
    /// An error when there is an IO exception.
    #[error(transparent)]
    InvalidIo(#[from] std::io::Error),
    /// An error to represent errors with converting between data types.
    #[error("Conversion error, {0}")]
    Conversion(Cow<'static, str>),
    #[error("Parsing error, {0}")]
    /// An error to represent errors around parsing.
    Parsing(Cow<'static, str>),
    /// A generic error.
    #[error("source: {0}, reason: {1}")]
    Other(Cow<'static, str>, Cow<'static, str>),
}

impl CollectionError {
    /// A generic error.
    pub fn other<C: Into<Cow<'static, str>>, D: Into<Cow<'static, str>>>(
        source: C, reason: D,
    ) -> Self {
        Self::Other(source.into(), reason.into())
    }
}

impl From<Utf8Error> for CollectionError {
    fn from(err: Utf8Error) -> Self {
        CollectionError::Conversion(err.to_string().into())
    }
}

impl From<ParseFloatError> for CollectionError {
    fn from(err: ParseFloatError) -> Self {
        CollectionError::Parsing(err.to_string().into())
    }
}
