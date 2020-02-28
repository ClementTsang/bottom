use std::result;

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
}

impl std::fmt::Display for BottomError {
	fn fmt(&self, f : &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			BottomError::InvalidIO(ref message) => {
				write!(f, "Encountered an IO exception: {}", message)
			}
			BottomError::InvalidArg(ref message) => write!(f, "Invalid argument: {}", message),
			BottomError::InvalidHeim(ref message) => write!(
				f,
				"Invalid error during data collection due to Heim: {}",
				message
			),
			BottomError::CrosstermError(ref message) => {
				write!(f, "Invalid error due to Crossterm: {}", message)
			}
			BottomError::GenericError(ref message) => write!(f, "{}", message),
			BottomError::FernError(ref message) => write!(f, "Invalid fern error: {}", message),
			BottomError::ConfigError(ref message) => {
				write!(f, "Invalid config file error: {}", message)
			}
		}
	}
}

impl From<std::io::Error> for BottomError {
	fn from(err : std::io::Error) -> Self {
		BottomError::InvalidIO(err.to_string())
	}
}

impl From<heim::Error> for BottomError {
	fn from(err : heim::Error) -> Self {
		BottomError::InvalidHeim(err.to_string())
	}
}

impl From<crossterm::ErrorKind> for BottomError {
	fn from(err : crossterm::ErrorKind) -> Self {
		BottomError::CrosstermError(err.to_string())
	}
}

impl From<std::num::ParseIntError> for BottomError {
	fn from(err : std::num::ParseIntError) -> Self {
		BottomError::InvalidArg(err.to_string())
	}
}

impl From<std::string::String> for BottomError {
	fn from(err : std::string::String) -> Self {
		BottomError::GenericError(err)
	}
}

impl From<toml::de::Error> for BottomError {
	fn from(err : toml::de::Error) -> Self {
		BottomError::ConfigError(err.to_string())
	}
}

impl From<fern::InitError> for BottomError {
	fn from(err : fern::InitError) -> Self {
		BottomError::FernError(err.to_string())
	}
}
