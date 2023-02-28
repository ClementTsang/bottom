use std::{borrow::Cow, result};

use thiserror::Error;

#[cfg(target_os = "linux")]
use procfs::ProcError;

/// A type alias for handling errors related to Bottom.
pub type Result<T> = result::Result<T, BottomError>;

/// An error that can occur while Bottom runs.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BottomError {
    /// An error when there is an IO exception.
    #[error("IO exception, {0}")]
    InvalidIo(String),
    /// An error when the heim library encounters a problem.
    #[error("Error caused by Heim, {0}")]
    InvalidHeim(String),
    /// An error when the Crossterm library encounters a problem.
    #[error("Error caused by Crossterm, {0}")]
    CrosstermError(String),
    /// An error to represent generic errors.
    #[error("Error, {0}")]
    GenericError(String),
    /// An error to represent errors with fern.
    #[error("Fern error, {0}")]
    FernError(String),
    /// An error to represent errors with the config.
    #[error("Configuration file error, {0}")]
    ConfigError(String),
    /// An error to represent errors with converting between data types.
    #[error("Conversion error, {0}")]
    ConversionError(String),
    /// An error to represent errors with querying.
    #[error("Query error, {0}")]
    QueryError(Cow<'static, str>),
    /// An error that just signifies something minor went wrong; no message.
    #[error("Minor error.")]
    MinorError,
    /// An error to represent errors with procfs
    #[cfg(target_os = "linux")]
    #[error("Procfs error, {0}")]
    ProcfsError(String),
}

impl From<std::io::Error> for BottomError {
    fn from(err: std::io::Error) -> Self {
        BottomError::InvalidIo(err.to_string())
    }
}

#[cfg(not(target_os = "freebsd"))]
impl From<heim::Error> for BottomError {
    fn from(err: heim::Error) -> Self {
        BottomError::InvalidHeim(err.to_string())
    }
}

impl From<std::num::ParseIntError> for BottomError {
    fn from(err: std::num::ParseIntError) -> Self {
        BottomError::ConfigError(err.to_string())
    }
}

impl From<std::string::String> for BottomError {
    fn from(err: std::string::String) -> Self {
        BottomError::GenericError(err)
    }
}

impl From<toml_edit::de::Error> for BottomError {
    fn from(err: toml_edit::de::Error) -> Self {
        BottomError::ConfigError(err.to_string())
    }
}

#[cfg(feature = "fern")]
impl From<fern::InitError> for BottomError {
    fn from(err: fern::InitError) -> Self {
        BottomError::FernError(err.to_string())
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

impl From<regex::Error> for BottomError {
    fn from(err: regex::Error) -> Self {
        // We only really want the last part of it... so we'll do it the ugly way:
        let err_str = err.to_string();
        let error = err_str.split('\n').map(|s| s.trim()).collect::<Vec<_>>();

        BottomError::QueryError(format!("Regex error: {}", error.last().unwrap_or(&"")).into())
    }
}

#[cfg(target_os = "linux")]
impl From<ProcError> for BottomError {
    fn from(err: ProcError) -> Self {
        match err {
            ProcError::PermissionDenied(p) => {
                BottomError::ProcfsError(format!("Permission denied for {:?}", p))
            }
            ProcError::NotFound(p) => BottomError::ProcfsError(format!("{:?} not found", p)),
            ProcError::Incomplete(p) => BottomError::ProcfsError(format!("{:?} incomplete", p)),
            ProcError::Io(e, p) => {
                BottomError::ProcfsError(format!("io error: {:?} for {:?}", e, p))
            }
            ProcError::Other(s) => BottomError::ProcfsError(format!("Other procfs error: {}", s)),
            ProcError::InternalError(e) => {
                BottomError::ProcfsError(format!("procfs internal error: {:?}", e))
            }
        }
    }
}
