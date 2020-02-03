use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    /// Invalid arguments provided.
    InvalidArguments(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidArguments(ref msg) => write!(f, "Invalid arguments: {}", msg),
        }
    }
}

impl error::Error for Error {}
