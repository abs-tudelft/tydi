//! Generator methods and implementations for Tydi types.
//!
//! The generator module is enabled by the `generator` feature flag.

use std::{error::Error, path::Path};

use crate::generator::common::Project;

pub mod chisel;
pub mod common;
pub mod vhdl;

/// Trait to generate back-end specific source files from the common hardware representation
/// of a project.
pub trait GenerateProject {
    /// Generate source files from a [common::Project] and save them to [path].
    fn generate(&self, project: &Project, path: &Path) -> Result<(), Box<dyn Error>>;
}
