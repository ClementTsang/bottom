use anyhow::anyhow;

/// An error to do with data collection.
#[derive(Debug)]
pub enum CollectionError {
    /// A general error to propagate back up. A wrapper around [`anyhow::Error`].
    General(anyhow::Error),

    /// The collection is unsupported.
    Unsupported,
}

impl CollectionError {
    // pub(crate) fn general<E: Into<anyhow::Error>>(error: E) -> Self {
    //     Self::General(error.into())
    // }

    pub(crate) fn from_str(msg: &'static str) -> Self {
        Self::General(anyhow!(msg))
    }
}

/// A [`Result`] with the error type being a [`DataCollectionError`].
pub(crate) type CollectionResult<T> = Result<T, CollectionError>;

impl From<std::io::Error> for CollectionError {
    fn from(err: std::io::Error) -> Self {
        CollectionError::General(err.into())
    }
}
