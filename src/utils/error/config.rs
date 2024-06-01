//! Error code related to configuration.

use std::borrow::Cow;

use thiserror::Error;

/// A type alias for handling collection-related errors.
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// The errors that can happen with data collection.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// An error when there is an IO exception.
    #[error(transparent)]
    InvalidIo(#[from] std::io::Error),
    /// An error due to parsing.
    #[error("Parsing error: {0}")]
    Parsing(Cow<'static, str>),
    /// A generic error.
    #[error("{0}")]
    Other(Cow<'static, str>),
}

impl ConfigError {
    /// A generic error.
    pub fn other<C: Into<Cow<'static, str>>>(source: C) -> Self {
        Self::Other(source.into())
    }
}

impl From<toml_edit::de::Error> for ConfigError {
    fn from(err: toml_edit::de::Error) -> Self {
        ConfigError::Parsing(err.to_string().into())
    }
}

impl From<std::str::Utf8Error> for ConfigError {
    fn from(err: std::str::Utf8Error) -> Self {
        ConfigError::Parsing(err.to_string().into())
    }
}

impl From<std::string::FromUtf8Error> for ConfigError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ConfigError::Parsing(err.to_string().into())
    }
}
