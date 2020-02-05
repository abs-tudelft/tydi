//! Error variants.

use std::{error, fmt};

/// Error variants used in this crate.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Indicates an invalid argument is provided.
    InvalidArgument(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidArgument(ref msg) => write!(f, "Invalid argument: {}", msg),
        }
    }
}

impl error::Error for Error {}
