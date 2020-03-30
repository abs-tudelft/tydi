//! Error variants.
use log::SetLoggerError;
use nom::lib::std::convert::Infallible;
use std::{error, fmt, result};

/// Result type with [`Error`] variants.
///
/// [`Error`]: ./enum.Error.html
pub type Result<T> = result::Result<T, Error>;

/// Error variants used in this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Unknown error.
    UnknownError,
    /// Errors related to the command-line interface.
    CLIError(String),
    /// Indicates an invalid argument is provided.
    InvalidArgument(String),
    /// Indicates an unexpected duplicate is provided.
    UnexpectedDuplicate,
    /// Errors related to the file system.
    FileIOError(String),
    /// Errors related to the parser.
    ParsingError(String),
    /// Errors related to back-ends.
    BackEndError(String),
    /// Errors related to interfaces.
    InterfaceError(String),
    /// Errors related to libraries.
    ProjectError(String),
    /// Errors related to implementation.
    ImplementationError(String),
}

impl fmt::Display for Error {
    /// Display the error variants.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CLIError(ref msg) => write!(f, "CLI Error: {}", msg),
            Error::InvalidArgument(ref msg) => write!(f, "Invalid argument: {}", msg),
            Error::UnexpectedDuplicate => write!(f, "Unexpected duplicate"),
            Error::UnknownError => write!(f, "Unknown error"),
            Error::FileIOError(ref msg) => write!(f, "File I/O error: {}", msg),
            Error::ParsingError(ref msg) => write!(f, "Parsing error: {}", msg),
            Error::BackEndError(ref msg) => write!(f, "Back-end error: {}", msg),
            Error::InterfaceError(ref msg) => write!(f, "Interface error: {}", msg),
            Error::ProjectError(ref msg) => write!(f, "Composition error: {}", msg),
            Error::ImplementationError(ref msg) => write!(f, "Composition error: {}", msg),
        }
    }
}

impl error::Error for Error {}

impl From<Box<dyn error::Error>> for Error {
    fn from(error: Box<dyn error::Error>) -> Self {
        if let Ok(error) = error.downcast::<Self>() {
            *error
        } else {
            Error::UnknownError
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileIOError(e.to_string())
    }
}

impl From<SetLoggerError> for Error {
    fn from(e: SetLoggerError) -> Self {
        Error::CLIError(e.to_string())
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error() {
        let a = Error::InvalidArgument("test".to_string());
        let b = Error::UnexpectedDuplicate;
        assert_eq!(a.to_string(), "Invalid argument: test");
        assert_eq!(b.to_string(), "Unexpected duplicate");
    }
}
