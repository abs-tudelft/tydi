//! VHDL back-end.
//!
//! This module contains functionality to convert hardware defined in the common hardware
//! representation to VHDL source files.

use std::error::Error;
use std::path::Path;

use crate::generator::common::*;
use crate::generator::GenerateProject;

mod impls;

/// VHDL back-end code generation result
type VHDLResult = Result<String, VHDLError>;

/// VHDL back-end errors.
#[derive(Debug, Clone, PartialEq)]
pub enum VHDLError {
    NotSynthesizable,
    TypeNameConflict,
}

impl std::fmt::Display for VHDLError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".")
    }
}

impl std::error::Error for VHDLError {}

/// Generate trait for VHDL declarations.
pub trait Declare {
    /// Generate a VHDL declaration from self.
    fn declare(&self) -> VHDLResult;
}

/// Generate trait for VHDL identifiers.
pub trait Identify {
    /// Generate a VHDL identifier from self.
    fn identify(&self) -> VHDLResult;
}

/// Analyze trait for VHDL objects.
pub trait Analyze {
    /// List all record types used.
    fn list_record_types(&self) -> Vec<Type>;
}

/// VHDL back-end configuration parameters.
pub struct VHDLConfig {
    /// An optional suffix appended to generated files.
    /// The suffix is added as follows: <filename>.<suffix>.vhd
    gen_suffix: Option<String>,
}

impl Default for VHDLConfig {
    fn default() -> Self {
        VHDLConfig {
            gen_suffix: Some("gen".to_string()),
        }
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
pub struct VHDLBackEnd {
    /// Configuration for the VHDL back-end.
    config: VHDLConfig,
}

impl GenerateProject for VHDLBackEnd {
    fn generate(&self, project: &Project, path: &Path) -> Result<(), Box<dyn Error>> {
        // Create the project directory.
        let mut dir = path.to_path_buf();
        dir.push(project.identifier.clone());
        std::fs::create_dir_all(dir.as_path())?;

        for lib in project.libraries.iter() {
            let mut pkg = dir.clone();
            pkg.push(format!("{}_pkg", lib.identifier));
            pkg.set_extension(match self.config.gen_suffix.clone() {
                None => "vhd".to_string(),
                Some(suffix) => format!("{}.vhd", suffix),
            });
            std::fs::write(pkg.as_path(), lib.declare()?)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generator::common::test::*;

    #[test]
    fn test_type_conflict() {
        let t0 = Type::record("a", vec![Field::new("x", Type::Bit)]);
        let t1 = Type::record("a", vec![Field::new("y", Type::Bit)]);
        let c = Component {
            identifier: "test".to_string(),
            parameters: vec![],
            ports: vec![Port::new("q", Mode::In, t0), Port::new("r", Mode::Out, t1)],
        };
        let p = Library {
            identifier: "lib".to_string(),
            components: vec![c],
        };
        let result = p.declare();
        assert_eq!(result.unwrap_err(), VHDLError::TypeNameConflict);
    }

    #[test]
    fn test_backend() {
        let v = VHDLBackEnd::default();
        assert!(v.generate(&test_proj(), Path::new("__test")).is_ok());

        // Check if files were correclty generated.
        assert!(std::fs::metadata("__test").is_ok());
        assert!(std::fs::metadata("__test/proj").is_ok());
        assert!(std::fs::metadata("__test/proj/lib_pkg.gen.vhd").is_ok());

        // Remove everything that was generated.
        assert!(std::fs::remove_dir_all(Path::new("__test")).is_ok());
    }
}
