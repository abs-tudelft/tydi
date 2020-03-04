//! Generator methods and implementations for Tydi types.
//!
//! The generator module is enabled by the `generator` feature flag.

use std::path::Path;

use crate::generator::common::{Component, Library, Mode, Port, Project, Record, Type};
use crate::physical::PhysicalStream;
use crate::traits::Identify;
use crate::Result;

pub mod chisel;
pub mod common;
pub mod vhdl;

/// Trait to generate back-end specific source files from the common hardware representation
/// of a project.
pub trait GenerateProject {
    /// Generate source files from a [common::Project] and save them to [path].
    fn generate(&self, project: &Project, path: &Path) -> Result<()>;
}

pub trait Synthesize {
    fn synthesize(&self, prefix: impl Into<String>) -> Type;
}

impl Synthesize for PhysicalStream {
    fn synthesize(&self, prefix: impl Into<String>) -> Type {
        // Creates a common HW representation type from a physical `Stream` according to
        // Tydi specification.
        let mut rec = Record::empty(prefix.into());

        // Valid/Ready handshake ports
        rec.add_field("valid", Type::Bit);
        rec.add_field_rev("ready", Type::Bit);

        // Iterate over signal map.
        for (name, width) in self.signal_map().into_iter() {
            rec.add_field(name, Type::bitvec(width));
        }

        Type::Record(rec)
    }
}

impl From<crate::design::Mode> for Mode {
    fn from(m: crate::design::Mode) -> Self {
        match m {
            crate::design::Mode::Out => Mode::Out,
            crate::design::Mode::In => Mode::In,
        }
    }
}

impl From<crate::design::Interface> for Vec<Port> {
    fn from(i: crate::design::Interface) -> Self {
        i.typ()
            .synthesize()
            .streams()
            .map(|(n, p)| {
                Port::new(
                    format!("{}{}", i.name(), n.to_string()),
                    i.mode().into(),
                    p.synthesize(format!("{}_{}", i.name(), "rec")),
                )
            })
            .collect()
    }
}

impl From<crate::design::Streamlet> for Component {
    fn from(s: crate::design::Streamlet) -> Self {
        Component {
            identifier: s.name().to_string(),
            parameters: vec![],
            ports: {
                let mut ports = Vec::new();
                s.interfaces().into_iter().for_each(|i| {
                    let vp: Vec<Port> = i.into();
                    ports.extend(vp);
                });
                ports
            },
        }
    }
}

impl From<crate::design::Library> for Library {
    fn from(l: crate::design::Library) -> Self {
        Library {
            identifier: l.name().to_string(),
            components: l.streamlets().into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl From<crate::design::Project> for Project {
    fn from(p: crate::design::Project) -> Self {
        Project {
            identifier: p.name().to_string(),
            libraries: p.libraries().into_iter().map(|l| l.into()).collect(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::design::{Interface, Streamlet};
    use crate::generator::common::{Component, Mode, Port};
    use crate::generator::vhdl::Declare;
    use crate::generator::Synthesize;
    use crate::physical::PhysicalStream;
    use crate::{Name, Result, UniquelyNamedBuilder};

    #[test]
    pub(crate) fn simple_streamlet() -> Result<()> {
        let streamlet = Streamlet::from_builder(
            Name::try_new("test")?,
            UniquelyNamedBuilder::new().with_items(vec![
                Interface::try_new(
                    "x",
                    crate::design::Mode::In,
                    crate::logical::tests::streams::single_element(),
                )?,
                Interface::try_new(
                    "y",
                    crate::design::Mode::Out,
                    crate::logical::tests::streams::single_element(),
                )?,
            ]),
        )?;
        // TODO(johanpel): write actual test
        dbg!(&streamlet);
        let common_streamlet: Component = streamlet.into();
        dbg!(&common_streamlet);
        println!("{}", common_streamlet.declare()?);
        Ok(())
    }

    #[test]
    fn interface_to_canonical() -> Result<()> {
        let interface = Interface::try_new(
            "x",
            crate::design::Mode::Out,
            crate::logical::tests::streams::single_element(),
        )?;
        let ports: Vec<Port> = interface.into();
        // TODO(johanpel): write actual test
        dbg!(ports);
        Ok(())
    }

    #[test]
    fn physical_low_complexity() -> Result<()> {
        let phys = PhysicalStream::try_new(vec![("a", 4), ("b", 8)], 2, 0, 2, vec![])?;
        let common_type = phys.synthesize("test");
        // TODO(johanpel): write actual test
        println!("{}", common_type.declare().unwrap());
        Ok(())
    }

    #[test]
    fn physical_high_complexity() -> Result<()> {
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
        // TODO(johanpel): write actual test
        println!("{}", comp.declare().unwrap());
        Ok(())
    }
}
