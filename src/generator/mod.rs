//! Generator methods and implementations for Tydi types.
//!
//! The generator module is enabled by the `generator` feature flag.

use std::{error::Error, path::Path};

use crate::generator::common::{Project, Record, Type};
use crate::physical::PhysicalStream;

pub mod chisel;
pub mod common;
pub mod vhdl;

/// Trait to generate back-end specific source files from the common hardware representation
/// of a project.
pub trait GenerateProject {
    /// Generate source files from a [common::Project] and save them to [path].
    fn generate(&self, project: &Project, path: &Path) -> Result<(), Box<dyn Error>>;
}

pub trait Synthesize {
    fn synthesize(&self, prefix: impl Into<String>) -> Type;
}

impl Synthesize for PhysicalStream {
    fn synthesize(&self, prefix: impl Into<String>) -> Type {
        // Creates a type from a physical `Stream` according to Tidy spec.
        let mut rec = Record::empty(prefix.into());

        // Valid/Ready handshake ports
        rec.add_field("valid", Type::Bit);
        rec.add_field_rev("ready", Type::Bit);

        for (name, width) in self.signal_map().into_iter() {
            rec.add_field(name, Type::bitvec(width));
        }

        Type::Record(rec)
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::common::{Component, Mode, Port};
    use crate::generator::vhdl::Declare;
    use crate::generator::Synthesize;
    use crate::physical::PhysicalStream;
    use crate::Result;

    #[test]
    fn test_synth_simple() -> Result<()> {
        let phys = PhysicalStream::try_new(vec![("a", 4), ("b", 8)], 2, 0, 2, vec![])?;
        let common_type = phys.synthesize("test");
        println!("{}", common_type.declare().unwrap());
        Ok(())
    }

    #[test]
    fn test_synth_complex() -> Result<()> {
        let phys = PhysicalStream::try_new(
            vec![("a", 4), ("b", 8)],
            4,
            3,
            8,
            vec![("muh", 3), ("foo", 4)],
        )?;

        let common_type = phys.synthesize("test");

        let mut comp = Component {
            identifier: "MyComp".to_string(),
            parameters: vec![],
            ports: vec![Port::new("x", Mode::In, common_type)],
        };

        comp.flatten_types();

        println!("{}", comp.declare().unwrap());
        Ok(())
    }
}
