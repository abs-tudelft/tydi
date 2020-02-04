//! Streamlet definition.

use crate::LogicalStream;

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    pub identifier: String,
    pub inputs: Vec<LogicalStream>,
    pub outputs: Vec<LogicalStream>,
}
