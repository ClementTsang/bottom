use std::{borrow::Cow, result};

/// A type alias for handling errors related to Bottom.
pub type Result<T> = result::Result<T, BottomError>;

/// An error that can occur while Bottom runs.
#[derive(Debug)]
pub enum BottomError {
    /// An error when there is an IO exception.
    InvalidIO(String),
    /// An error when there is an invalid argument passed in.
    InvalidArg(String),
    /// An error when the heim library encounters a problem.
    InvalidHeim(String),
    /// An error when the Crossterm library encounters a problem.
    CrosstermError(String),
    /// An error to represent generic errors.
    GenericError(String),
    /// An error to represent errors with fern.
    FernError(String),
    /// An error to represent errors with the config.
    ConfigError(String),
    /// An error to represent errors with converting between data types.
    ConversionError(String),
    /// An error to represent errors with querying.
    QueryError(Cow<'static, str>),
}

impl std::fmt::Display for BottomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BottomError::InvalidIO(ref message) => {
                write!(f, "encountered an IO exception: {}", message)
            }
            BottomError::InvalidArg(ref message) => write!(f, "Invalid argument: {}", message),
            BottomError::InvalidHeim(ref message) => write!(
                f,
                "invalid error during data collection due to heim: {}",
                message
            ),
            BottomError::CrosstermError(ref message) => {
                write!(f, "invalid error due to Crossterm: {}", message)
            }
            BottomError::GenericError(ref message) => write!(f, "{}", message),
            BottomError::FernError(ref message) => write!(f, "Invalid fern error: {}", message),
            BottomError::ConfigError(ref message) => {
                write!(f, "invalid config file error: {}", message)
            }
            BottomError::ConversionError(ref message) => {
                write!(f, "unable to convert: {}", message)
            }
            BottomError::QueryError(ref message) => write!(f, "{}", message),
        }
    }
}

impl From<std::io::Error> for BottomError {
    fn from(err: std::io::Error) -> Self {
        BottomError::InvalidIO(err.to_string())
    }
}

impl From<heim::Error> for BottomError {
    fn from(err: heim::Error) -> Self {
        BottomError::InvalidHeim(err.to_string())
    }
}

impl From<crossterm::ErrorKind> for BottomError {
    fn from(err: crossterm::ErrorKind) -> Self {
        BottomError::CrosstermError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for BottomError {
    fn from(err: std::num::ParseIntError) -> Self {
        BottomError::InvalidArg(err.to_string())
    }
}

impl From<std::string::String> for BottomError {
    fn from(err: std::string::String) -> Self {
        BottomError::GenericError(err)
    }
}

impl From<toml::de::Error> for BottomError {
    fn from(err: toml::de::Error) -> Self {
        BottomError::ConfigError(err.to_string())
    }
}

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

impl From<regex::Error> for BottomError {
    fn from(err: regex::Error) -> Self {
        // We only really want the last part of it... so we'll do it the ugly way:
        let err_str = err.to_string();
        let error = err_str.split('\n').map(|s| s.trim()).collect::<Vec<_>>();

        BottomError::QueryError(
            format!(
                "Regex error: {}",
                error.last().unwrap_or(&"".to_string().as_str())
            )
            .into(),
        )
    }
}
