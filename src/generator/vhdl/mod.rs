//! VHDL back-end.
//!
//! This module contains functionality to convert hardware defined in the common hardware
//! representation to VHDL source files.

use crate::generator::common::*;
use crate::generator::GenerateProject;
use crate::{Error, Result};
use log::debug;
use std::path::Path;

use std::str::FromStr;
#[cfg(feature = "cli")]
use structopt::StructOpt;

mod impls;

/// Generate trait for VHDL declarations.
pub trait Declare {
    /// Generate a VHDL declaration from self.
    fn declare(&self) -> Result<String>;
}

/// Generate trait for VHDL identifiers.
pub trait Identify {
    /// Generate a VHDL identifier from self.
    fn identify(&self) -> Result<String>;
}

/// Analyze trait for VHDL objects.
pub trait Analyze {
    /// List all record types used.
    fn list_record_types(&self) -> Vec<Type>;
}

/// Abstraction levels
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub enum AbstractionLevel {
    Canonical,
    Fancy,
}

impl FromStr for AbstractionLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "canonical" => Ok(AbstractionLevel::Canonical),
            "fancy" => Ok(AbstractionLevel::Fancy),
            _ => Err(Error::InvalidArgument(s.to_string())),
        }
    }
}

/// VHDL back-end configuration parameters.
#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub struct VHDLConfig {
    /// Abstraction level of generated files.
    /// Possible options: canonical, fancy.
    ///   canonical: generates the canonical Tydi representation of streamlets as components in a
    ///              package.
    ///   fancy: generates the canonical components that wrap a more user-friendly version for the
    ///          user to implement.
    #[structopt(short, long)]
    abstraction: Option<AbstractionLevel>,

    /// Suffix of generated files. Default = "gen", such that
    /// generated files are named <name>.gen.vhd.
    #[structopt(short, long)]
    suffix: Option<String>,
}

impl Default for VHDLConfig {
    fn default() -> Self {
        VHDLConfig {
            suffix: Some("gen".to_string()),
            abstraction: Some(AbstractionLevel::Canonical),
        }
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
pub struct VHDLBackEnd {
    /// Configuration for the VHDL back-end.
    config: VHDLConfig,
}

impl From<VHDLConfig> for VHDLBackEnd {
    fn from(config: VHDLConfig) -> Self {
        VHDLBackEnd { config }
    }
}

impl GenerateProject for VHDLBackEnd {
    fn generate(&self, project: &Project, path: &Path) -> Result<()> {
        // Create the project directory.
        let mut dir = path.to_path_buf();
        dir.push(project.identifier.clone());
        std::fs::create_dir_all(dir.as_path())?;

        for lib in project.libraries.iter() {
            let mut pkg = dir.clone();
            pkg.push(format!("{}_pkg", lib.identifier));
            pkg.set_extension(match self.config.suffix.clone() {
                None => "vhd".to_string(),
                Some(s) => format!("{}.vhd", s),
            });
            std::fs::write(pkg.as_path(), lib.declare()?)?;
            debug!("Wrote {}.", pkg.as_path().to_str().unwrap_or(""));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generator::common::test::*;
    use std::fs;

    #[test]
    fn test_type_conflict() {
        let t0 = Type::record("a", vec![Field::new("x", Type::Bit, false)]);
        let t1 = Type::record("a", vec![Field::new("y", Type::Bit, false)]);
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
        // TODO(johanpel): make sure this tests for the right error:
        assert!(result.is_err());
    }

    #[test]
    fn test_backend() -> Result<()> {
        let v = VHDLBackEnd::default();

        let tmpdir = tempfile::tempdir()?;
        let path = tmpdir.path().join("__test");

        assert!(v.generate(&test_proj(), &path).is_ok());

        // Check if files were correctly generated.
        assert!(fs::metadata(&path).is_ok());
        assert!(fs::metadata(&path.join("proj")).is_ok());
        assert!(fs::metadata(&path.join("proj/lib_pkg.gen.vhd")).is_ok());

        Ok(())
    }
}
