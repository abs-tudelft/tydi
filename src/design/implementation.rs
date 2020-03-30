//! Structures to support the implementation of streamlets.

use crate::design::structural::StructuralImpl;
use crate::design::StreamletRef;

/// An implementation variant.
#[derive(Debug, Clone, PartialEq)]
pub enum Implementation {
    None,
    Structural(StructuralImpl),
}

impl Implementation {
    /// Returns a reference to the streamlet this implementation implements.
    pub fn streamlet(&self) -> Option<StreamletRef> {
        match self {
            Implementation::None => None,
            Implementation::Structural(s) => Some(s.streamlet()),
        }
    }
}
