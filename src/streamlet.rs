//! Streamlet definition.

use crate::logical::LogicalStreamType;
use crate::Name;
use indexmap::IndexMap;

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    name: String,
    input: IndexMap<Name, LogicalStreamType>,
    output: IndexMap<Name, LogicalStreamType>,
}

impl Streamlet {
    pub fn new(
        name: impl Into<String>,
        input: impl IntoIterator<Item = (Name, LogicalStreamType)>,
        output: impl IntoIterator<Item = (Name, LogicalStreamType)>,
    ) -> Self {
        // todo result duplicate
        Streamlet {
            name: name.into(),
            input: input.into_iter().collect(),
            output: output.into_iter().collect(),
        }
    }
}
