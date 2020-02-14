//! Error variants.

use std::{error, fmt, result};

/// Result type with [`Error`] variants.
///
/// [`Error`]: ./enum.Error.html
pub type Result<T> = result::Result<T, Error>;

/// Error variants used in this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Indicates an invalid argument is provided.
    InvalidArgument(String),
    /// Indicates an unexpected duplicate is provided.
    UnexpectedDuplicate,
}

impl fmt::Display for Error {
    /// Display the error variants.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidArgument(ref msg) => write!(f, "Invalid argument: {}", msg),
            Error::UnexpectedDuplicate => write!(f, "Unexpected duplicate"),
        }
    }
}

impl error::Error for Error {}

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
