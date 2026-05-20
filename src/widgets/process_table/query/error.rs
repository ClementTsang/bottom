use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub(crate) struct QueryError {
    reason: Cow<'static, str>,
}

impl QueryError {
    #[inline]
    pub(crate) fn new<I: Into<Cow<'static, str>>>(reason: I) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    #[inline]
    pub(super) fn missing_value() -> Self {
        Self {
            reason: "Missing value".into(),
        }
    }
}

impl Display for QueryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl From<regex::Error> for QueryError {
    fn from(err: regex::Error) -> Self {
        Self::new(err.to_string())
    }
}

pub(super) type QueryResult<T> = Result<T, QueryError>;
