//! Chisel back-end.

use std::{error::Error, path::Path};

use crate::generator::{common::*, GenerateProject};

mod impls;

/// Chisel back-end errors.
#[derive(Debug, Clone, PartialEq)]
pub enum ChiselError {
    NotSynthesizable,
    TypeNameConflict,
}

impl std::fmt::Display for ChiselError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, ".")
    }
}

impl std::error::Error for ChiselError {}

/// Chisel back-end code generation result
#[allow(dead_code)]
type ChiselResult = Result<String, ChiselError>;

/// Chisel back-end configuration parameters.
pub struct ChiselConfig {
    /// An optional suffix appended to generated files.
    /// The suffix is added as follows: <filename>.<suffix>.scala
    #[allow(dead_code)]
    gen_suffix: Option<String>,
}

/// Generate trait for VHDL declarations.
pub trait Declare {
    /// Generate a VHDL declaration from self.
    fn declare(&self) -> ChiselResult;
}

/// Generate trait for VHDL identifiers.
pub trait Identify {
    /// Generate a VHDL identifier from self.
    fn identify(&self) -> ChiselResult;
}

/// Analyze trait for VHDL objects.
pub trait Analyze {
    /// List all record types used.
    fn list_record_types(&self) -> Vec<Type>;
}

impl Default for ChiselConfig {
    fn default() -> Self {
        ChiselConfig {
            gen_suffix: Some("gen".to_string()),
        }
    }
}

/// A configurable Chisel back-end entry point.
#[derive(Default)]
#[allow(dead_code)]
pub struct ChiselBackEnd {
    /// Configuration for the VHDL back-end.
    config: ChiselConfig,
}

#[allow(unused_variables)]
impl GenerateProject for ChiselBackEnd {
    fn generate(&self, project: &Project, path: &Path) -> Result<(), Box<dyn Error>> {
        // Create the project directory.
        let mut dir = path.to_path_buf();
        dir.push(project.identifier.clone());
        std::fs::create_dir_all(dir.as_path())?;

        for lib in project.libraries.iter() {
            let mut pkg = dir.clone();
            pkg.push(lib.identifier.clone());
            std::fs::create_dir_all(pkg.as_path())?;
            pkg.push(lib.identifier.clone());
            //pkg.push(format!("{}_pkg", lib.identifier));
            pkg.set_extension(match self.config.gen_suffix.clone() {
                None => "scala".to_string(),
                Some(suffix) => format!("{}.scala", suffix),
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
    use std::{error, fs};

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
        //assert_eq!(result.unwrap_err(), VHDLError::TypeNameConflict);
    }

    #[test]
    fn test_backend() -> Result<(), Box<dyn error::Error>> {
        let v = ChiselBackEnd::default();

        //let tmpdir = tempfile::tempdir()?;
        //let path = tmpdir.path().join("__test");

        let path = Path::new("/home/akos/temp");

        v.generate(&test_proj(), &path);

        // Check if files were correclty generated.
        //assert!(fs::metadata(&path).is_ok());
        //assert!(fs::metadata(&path.join("proj")).is_ok());
        //assert!(fs::metadata(&path.join("proj/lib_pkg.gen.vhd")).is_ok());

        Ok(())
    }
}
