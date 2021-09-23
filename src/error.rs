//! Error variants.
use std::{error, fmt, result};

use log::SetLoggerError;

/// Result type with [`Error`] variants.
///
/// [`Error`]: ./enum.Error.html
pub type Result<T> = result::Result<T, Error>;

/// Error variants used in this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Unknown error.
    UnknownError,
    /// Generic CLI error.
    CLIError(String),
    /// Indicates an invalid argument is provided.
    InvalidArgument(String),
    /// Indicates an unexpected duplicate is provided.
    UnexpectedDuplicate,
    /// File I/O error.
    FileIOError(String),
    /// Parsing error.
    ParsingError(String),
    /// Parsing error.
    ImplParsingError(LineErr),
    /// Invalid target.
    InvalidTarget(String),
    /// Back-end error.
    BackEndError(String),
    /// Forbidden interface name.
    InterfaceError(String),
    /// Project error
    ProjectError(String),
    /// Composer error
    ComposerError(String),
    /// Library error
    LibraryError(String),
}

///Error variants for implementation parser
#[derive(Debug, PartialEq, Clone)]
pub struct LineErr {
    pub line: usize,
    pub err: String,
}

impl LineErr {
    #[allow(dead_code)]
    pub(crate) fn new(l: usize, s: String) -> Self {
        LineErr { line: l, err: s }
    }
}

impl LineErr {
    pub fn on_line(self, n: usize) -> LineErr {
        LineErr {
            line: n,
            err: self.err,
        }
    }
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
            Error::ImplParsingError(ref err) => write!(
                f,
                "Implementation parsing error on line: {}:{}",
                err.line, err.err
            ),
            Error::InvalidTarget(ref msg) => write!(f, "Invalid target: {}", msg),
            Error::BackEndError(ref msg) => write!(f, "Back-end error: {}", msg),
            Error::InterfaceError(ref msg) => write!(f, "Interface error: {}", msg),
            Error::ProjectError(ref msg) => write!(f, "Project error: {}", msg),
            Error::ComposerError(ref msg) => write!(f, "Composer error: {}", msg),
            Error::LibraryError(ref msg) => write!(f, "Library error: {}", msg),
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
