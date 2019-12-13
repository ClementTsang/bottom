use failure::Fail;
use std::result;

/// A type alias for handling errors related to Bottom.
pub type Result<T> = result::Result<T, BottomError>;

/// An error that can occur while Bottom runs.
#[derive(Debug, Fail)]
pub enum BottomError {
	/// An error when there is an IO exception.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Encountered an IO exception: {}", message)]
	InvalidIO { message: String },
	/// An error when there is an invalid argument passed in.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid argument: {}", message)]
	InvalidArg { message: String },
	/// An error when the heim library encounters a problem.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid error during data collection due to Heim: {}", message)]
	InvalidHeim { message: String },
	/// An error when the Crossterm library encounters a problem.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid error due to Crossterm: {}", message)]
	CrosstermError { message: String },
	/// An error to represent generic errors
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid generic error: {}", message)]
	GenericError { message: String },
	/// An error to represent errors with fern
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid fern error: {}", message)]
	FernError { message: String },
}

impl From<std::io::Error> for BottomError {
	fn from(err: std::io::Error) -> Self {
		BottomError::InvalidIO { message: err.to_string() }
	}
}

impl From<heim::Error> for BottomError {
	fn from(err: heim::Error) -> Self {
		BottomError::InvalidHeim { message: err.to_string() }
	}
}

impl From<crossterm::ErrorKind> for BottomError {
	fn from(err: crossterm::ErrorKind) -> Self {
		BottomError::CrosstermError { message: err.to_string() }
	}
}

impl From<std::num::ParseIntError> for BottomError {
	fn from(err: std::num::ParseIntError) -> Self {
		BottomError::InvalidArg { message: err.to_string() }
	}
}

impl From<std::string::String> for BottomError {
	fn from(err: std::string::String) -> Self {
		BottomError::GenericError { message: err.to_string() }
	}
}

impl From<fern::InitError> for BottomError {
	fn from(err: fern::InitError) -> Self {
		BottomError::FernError { message: err.to_string() }
	}
}
