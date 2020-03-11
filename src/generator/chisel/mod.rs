//! Chisel back-end.

use crate::generator::{common::Project, GenerateProject};
use crate::Result;
use std::path::Path;

/// Chisel back-end code generation result
#[allow(dead_code)]
type ChiselResult = Result<String>;

/// Chisel back-end configuration parameters.
pub struct ChiselConfig {
    /// An optional suffix appended to generated files.
    /// The suffix is added as follows: <filename>.<suffix>.scala
    #[allow(dead_code)]
    gen_suffix: Option<String>,
}

impl Default for ChiselConfig {
    fn default() -> Self {
        ChiselConfig {
            gen_suffix: Some("gen".to_string()),
        }
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
#[allow(dead_code)]
pub struct ChiselBackEnd {
    /// Configuration for the VHDL back-end.
    config: ChiselConfig,
}

#[allow(unused_variables)]
impl GenerateProject for ChiselBackEnd {
    fn generate(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        unimplemented!();
    }
}
