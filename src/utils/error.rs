mod collection;
mod config;
mod draw;

use std::borrow::Cow;

pub use collection::*;
pub use config::*;
pub use draw::*;
use thiserror::Error;

/// A type alias for handling errors related to `bottom`.
pub type Result<T> = std::result::Result<T, BottomError>;

/// An error that can occur while `bottom` runs.
#[derive(Debug, Error)]
pub enum BottomError {
    /// An error regarding data collection.
    #[error(transparent)]
    Collection(CollectionError),
    /// An error regarding drawing.
    #[error(transparent)]
    Draw(DrawError),
    /// An error to represent errors with the configuration.
    #[error(transparent)]
    Config(ConfigError),
    /// An error to represent errors with converting between data types.
    #[error("Conversion error, {0}")]
    Conversion(Cow<'static, str>),
    #[error("{0}")]
    /// An error to report back to the user.
    User(Cow<'static, str>),
}

impl BottomError {
    /// An error related to configuration.
    pub fn config<C: Into<Cow<'static, str>>>(msg: C) -> Self {
        Self::Config(ConfigError::other(msg))
    }

    /// A user error.
    pub fn user<C: Into<Cow<'static, str>>>(msg: C) -> Self {
        Self::User(msg.into())
    }

    /// A error that arises from data harvesting.
    pub fn data_harvest<C: Into<Cow<'static, str>>, D: Into<Cow<'static, str>>>(
        source: C, reason: D,
    ) -> Self {
        Self::Collection(CollectionError::other(source.into(), reason.into()))
    }
}

impl From<CollectionError> for BottomError {
    fn from(err: CollectionError) -> Self {
        BottomError::Collection(err)
    }
}

impl From<DrawError> for BottomError {
    fn from(err: DrawError) -> Self {
        BottomError::Draw(err)
    }
}

impl From<std::str::Utf8Error> for BottomError {
    fn from(err: std::str::Utf8Error) -> Self {
        BottomError::Conversion(err.to_string().into())
    }
}

impl From<std::string::FromUtf8Error> for BottomError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BottomError::Conversion(err.to_string().into())
    }
}

impl From<std::num::TryFromIntError> for BottomError {
    fn from(err: std::num::TryFromIntError) -> Self {
        BottomError::Conversion(err.to_string().into())
    }
}

impl From<regex::Error> for BottomError {
    fn from(err: regex::Error) -> Self {
        // We only really want the last part of it.
        let err_str = err.to_string();
        let error = err_str.rsplit('\n').last().map(|s| s.trim()).unwrap_or("");

        BottomError::user(format!("Regex error: {error}"))
    }
}
