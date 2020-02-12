//! Streamlet definition.

use crate::logical::LogicalStream;

/// Streamlet interface definition.
#[derive(Clone, Debug)]
pub struct Streamlet {
    name: String,
    input: Vec<LogicalStream>,
    output: Vec<LogicalStream>,
}
