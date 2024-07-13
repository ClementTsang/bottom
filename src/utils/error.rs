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
    /// An error when the Crossterm library encounters a problem.
    #[error("Error caused by Crossterm, {0}")]
    CrosstermError(String),
    /// An error to represent generic errors.
    #[error("Error, {0}")]
    GenericError(String),
    /// An error to represent invalid command-line arguments.
    #[error("Invalid argument, {0}")]
    ArgumentError(String),
    /// An error to represent errors with the config.
    #[error("Configuration file error, {0}")]
    ConfigError(String),
    /// An error to represent errors with converting between data types.
    #[error("Conversion error, {0}")]
    ConversionError(String),
}

impl From<std::io::Error> for BottomError {
    fn from(err: std::io::Error) -> Self {
        BottomError::InvalidIo(err.to_string())
    }
}

impl From<std::num::ParseIntError> for BottomError {
    fn from(err: std::num::ParseIntError) -> Self {
        BottomError::ConfigError(err.to_string())
    }
}

impl From<String> for BottomError {
    fn from(err: String) -> Self {
        BottomError::GenericError(err)
    }
}

impl From<toml_edit::de::Error> for BottomError {
    fn from(err: toml_edit::de::Error) -> Self {
        BottomError::ConfigError(err.to_string())
    }
}

impl From<std::str::Utf8Error> for BottomError {
    fn from(err: std::str::Utf8Error) -> Self {
        BottomError::ConversionError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for BottomError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BottomError::ConversionError(err.to_string())
    }
}
