use anyhow::anyhow;

/// An error to do with data collection.
#[derive(Debug)]
pub enum CollectionError {
    /// A general error to propagate back up. A wrapper around [`anyhow::Error`].
    General(anyhow::Error),

    /// The collection is unsupported.
    Unsupported,
}

impl std::fmt::Display for CollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionError::General(err) => err.fmt(f),
            CollectionError::Unsupported => {
                write!(
                    f,
                    "bottom does not support this type of data collection for this platform."
                )
            }
        }
    }
}

impl std::error::Error for CollectionError {}

/// A [`Result`] with the error type being a [`DataCollectionError`].
pub(crate) type CollectionResult<T> = Result<T, CollectionError>;

impl From<std::io::Error> for CollectionError {
    fn from(err: std::io::Error) -> Self {
        Self::General(err.into())
    }
}

impl From<&'static str> for CollectionError {
    fn from(msg: &'static str) -> Self {
        Self::General(anyhow!(msg))
    }
}
