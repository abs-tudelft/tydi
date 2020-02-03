use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    /// Invalid argument provided.
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
