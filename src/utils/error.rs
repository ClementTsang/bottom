use failure::Fail;
use std::result;

/// A type alias for handling errors related to Rustop.
pub type Result<T> = result::Result<T, RustopError>;

/// An error that can occur while Rustop runs.
#[derive(Debug, Fail)]
pub enum RustopError {
	/// An error when there is an IO exception.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Encountered an IO exception: {}", message)]
	InvalidIO { message : String },
	/// An error when there is an invalid argument passed in.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid argument: {}", message)]
	InvalidArg { message : String },
	/// An error when the heim library encounters a problem.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid error during data collection due to Heim: {}", message)]
	InvalidHeim { message : String },
	/// An error when the Crossterm library encounters a problem.
	///
	/// The data provided is the error found.
	#[fail(display = "ERROR: Invalid error due to Crossterm: {}", message)]
	CrosstermError { message : String },
}

impl From<std::io::Error> for RustopError {
	fn from(err : std::io::Error) -> Self {
		RustopError::InvalidIO { message : err.to_string() }
	}
}

impl From<heim::Error> for RustopError {
	fn from(err : heim::Error) -> Self {
		RustopError::InvalidHeim { message : err.to_string() }
	}
}

impl From<crossterm::ErrorKind> for RustopError {
	fn from(err : crossterm::ErrorKind) -> Self {
		RustopError::CrosstermError { message : err.to_string() }
	}
}

impl From<std::num::ParseIntError> for RustopError {
	fn from(err : std::num::ParseIntError) -> Self {
		RustopError::InvalidArg { message : err.to_string() }
	}
}
