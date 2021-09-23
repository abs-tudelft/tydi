use std::path::Path;

use crate::design::Project;
use crate::Result;

pub mod chisel;
pub mod common;
pub mod vhdl;

/// Concatenate stuff using format with an underscore in between.
/// Useful if the separator ever changes.
#[macro_export]
macro_rules! cat {
    ($a:expr) => {{
        format!("{}", $a)
    }};

    ($a:expr, $($b:expr),+) => {{
        let left : String = format!("{}", $a);
        let right : String = format!("{}", cat!($($b),+));
        if left == "" {
            right
        } else if right == "" {
            left
        } else {
            format!("{}_{}", left, right)
        }
    }};
}

/// Trait to generate back-end specific source files from the common hardware representation
/// of a project.
pub trait GenerateProject {
    /// Generate source files from a [common::Project] and save them to [path].
    fn generate(&self, project: &Project, path: impl AsRef<Path>) -> Result<()>;
}
