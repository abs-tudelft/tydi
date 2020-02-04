use std::{error, fmt};

/// Error variants used in this crate.
#[derive(Debug)]
pub enum Error {
    /// Indicates an invalid argument is provided.
    InvalidArgument(String),
    /// Indicates an invalid identifier is provided.
    InvalidIdentifier(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidArgument(ref msg) => write!(f, "Invalid argument: {}", msg),
            Error::InvalidIdentifier(ref msg) => write!(f, "Invalid identifier: {}", msg),
        }
    }
}

impl error::Error for Error {}
