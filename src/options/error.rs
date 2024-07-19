use std::borrow::Cow;

/// An error around some option-setting, and the reason.
///
/// These are meant to potentially be user-facing (e.g. explain
/// why it's broken and what to fix), and as so treat it as such!
///
/// For stylistic and consistency reasons, use _single quotes_ (e.g. `'bad'`)
/// for highlighting error values. You can use (".*`.+`.*") as a regex to check
/// for this.
#[derive(Debug, PartialEq)]
pub enum OptionError {
    Config(Cow<'static, str>),
    Argument(Cow<'static, str>),
    Other(Cow<'static, str>),
}

impl OptionError {
    /// Create a new [`OptionError::Config`].
    pub(crate) fn config<R: Into<Cow<'static, str>>>(reason: R) -> Self {
        OptionError::Config(reason.into())
    }

    /// Create a new [`OptionError::Config`] for an invalid value.
    pub(crate) fn invalid_config_value(value: &str) -> Self {
        OptionError::Config(Cow::Owned(format!(
            "'{value}' was set with an invalid value, please update it in your config file."
        )))
    }

    /// Create a new [`OptionError::Argument`].
    pub(crate) fn arg<R: Into<Cow<'static, str>>>(reason: R) -> Self {
        OptionError::Argument(reason.into())
    }

    /// Create a new [`OptionError::Argument`] for an invalid value.
    pub(crate) fn invalid_arg_value(value: &str) -> Self {
        OptionError::Argument(Cow::Owned(format!(
            "'--{value}' was set with an invalid value, please update your arguments."
        )))
    }

    /// Create a new [`OptionError::Other`].
    pub(crate) fn other<R: Into<Cow<'static, str>>>(reason: R) -> Self {
        OptionError::Other(reason.into())
    }
}

pub(crate) type OptionResult<T> = Result<T, OptionError>;

impl std::fmt::Display for OptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionError::Config(reason) => write!(f, "Configuration file error: {reason}"),
            OptionError::Argument(reason) => write!(f, "Argument error: {reason}"),
            OptionError::Other(reason) => {
                write!(f, "Error with the config file or the arguments: {reason}")
            }
        }
    }
}

impl std::error::Error for OptionError {}

impl From<toml_edit::de::Error> for OptionError {
    fn from(err: toml_edit::de::Error) -> Self {
        OptionError::Config(err.to_string().into())
    }
}

impl From<std::io::Error> for OptionError {
    fn from(err: std::io::Error) -> Self {
        OptionError::Other(err.to_string().into())
    }
}
