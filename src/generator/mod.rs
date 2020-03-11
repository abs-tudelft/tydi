//! Generator methods and implementations for Tydi types.
//!
//! The generator module is enabled by the `generator` feature flag.

use std::path::Path;

use crate::design::{Interface, Streamlet};
use crate::generator::common::{Component, Library, Mode, Port, Project, Record, Type};
use crate::logical::{Group, LogicalStreamType, Stream};
use crate::physical::{Origin, Signal, Width};
use crate::traits::Identify;
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
    fn generate(&self, project: &Project, path: &Path) -> Result<()>;
}

/// Trait to create common representation types from things in the canonical
/// way and user-friendly way.
pub trait Typify {
    fn user(&self, _prefix: impl Into<String>) -> Option<Type> {
        None
    }
    fn canonical(&self, prefix: impl Into<String>) -> Vec<Signal>;
}

/// Trait to create common representation ports from things in the canonical
/// way and user-friendly way.
pub trait Portify {
    fn user(
        &self,
        _port_name: impl Into<String>,
        _port_type_prefix: impl Into<String>,
    ) -> Vec<Port> {
        Vec::new()
    }
    fn canonical(&self, name: impl Into<String>) -> Vec<Port>;
}

/// Trait to create common representation components from things in the canonical
/// way and user-friendly way.
pub trait Componentify {
    fn user(&self, _name: impl Into<String>) -> Option<Component> {
        None
    }
    fn canonical(&self, name: impl Into<String>) -> Component;
}

impl Typify for LogicalStreamType {
    /// This implementation for LogicalStreamType assumes the LogicalStreamType has already been
    /// flattened through synthesize.
    fn user(&self, prefix: impl Into<String>) -> Option<Type> {
        match self {
            LogicalStreamType::Bits(width) => Some(Type::bitvec(width.get())),
            LogicalStreamType::Group(group) => group.user(prefix),
            LogicalStreamType::Stream(stream) => stream.user(prefix),
            _ => todo!(),
        }
    }

    fn canonical(&self, prefix: impl Into<String>) -> Vec<Signal> {
        match self {
            LogicalStreamType::Bits(width) => {
                vec![Signal::vec(prefix.into(), Origin::Source, *width)]
            }
            LogicalStreamType::Group(group) => group.canonical(prefix),
            LogicalStreamType::Stream(stream) => stream.canonical(prefix),
            _ => todo!(),
        }
    }
}

impl Typify for Group {
    fn user(&self, prefix: impl Into<String>) -> Option<Type> {
        let n: String = prefix.into();
        let mut rec = Record::new_empty(cat!(n.clone(), "type"));
        for (field_name, field_logical) in self.iter() {
            if let Some(field_common_type) = field_logical.user(cat!(n.clone(), field_name)) {
                rec.insert_field(field_name, field_common_type, false)
            }
        }
        Some(Type::Record(rec))
    }

    fn canonical(&self, prefix: impl Into<String>) -> Vec<Signal> {
        let n: String = prefix.into();
        let mut result = Vec::new();
        for (field_name, field_logical) in self.iter() {
            let field_result = field_logical.canonical(cat!(n.clone(), field_name));
            result.extend(field_result);
        }
        result
    }
}

impl Typify for Stream {
    fn user(&self, prefix: impl Into<String>) -> Option<Type> {
        // We need to wrap the Stream back into a LogicalStreamType
        // to be able to use various methods for checks and synthesize.
        let logical = LogicalStreamType::from(self.clone());

        // At this point, it should not be possible that this is a
        // non-element-only LogicalStreamType.
        assert!(logical.is_element_only());

        // Check if the logical stream is null.
        if !logical.is_null() {
            // Synthesize the logical stream into physical streams.
            let synth = logical.synthesize();

            // Obtain the path name and signal map from the physical stream.
            // There should only be one, since it is an element only stream.
            // Therefore, it should be safe to unwrap.
            let (name, physical) = synth.streams().next().unwrap();
            let signals = physical.signal_list();

            // Set up the resulting record.
            let mut rec = Record::new_empty_stream(match name.len() {
                0 => cat!(prefix.into(), "type"),
                _ => cat!(prefix.into(), name, "type"),
            });

            // Insert data record. There must be something there since it is not null.
            rec.insert_field("data", self.data().user("data").unwrap(), false);

            // Check signals related to dimensionality, complexity, etc.
            if let Some(sig) = signals.last() {
                rec.insert_field("last", sig.width().into(), sig.reversed());
            }
            if let Some(sig) = signals.stai() {
                rec.insert_field("stai", sig.width().into(), sig.reversed());
            }
            if let Some(sig) = signals.endi() {
                rec.insert_field("endi", sig.width().into(), sig.reversed());
            }
            if let Some(sig) = signals.strb() {
                rec.insert_field("strb", sig.width().into(), sig.reversed());
            }

            Some(Type::Record(rec))
        } else {
            None
        }
    }

    fn canonical(&self, prefix: impl Into<String>) -> Vec<Signal> {
        let n: String = prefix.into();
        let mut result = Vec::new();

        let logical = LogicalStreamType::from(self.clone());
        assert!(logical.is_element_only());
        if !logical.is_null() {
            let synth = logical.synthesize();
            let (path, phys) = synth.streams().next().unwrap();
            for signal in phys.signal_list().into_iter() {
                let n = cat!(n.clone(), path, signal.identifier());
                result.push(signal.with_name(n));
            }
        }

        result
    }
}

impl From<Width> for Type {
    fn from(width: Width) -> Self {
        match width {
            Width::Scalar => Type::Bit,
            Width::Vector(w) => Type::bitvec(w),
        }
    }
}

/// Trait that helps to determine the common representation port mode given a streamlet interface
/// mode.
pub trait ModeFor {
    fn mode_for(&self, streamlet_mode: crate::design::Mode) -> Mode;
}

impl ModeFor for Origin {
    fn mode_for(&self, streamlet_mode: crate::design::Mode) -> Mode {
        match self {
            Origin::Sink => match streamlet_mode {
                crate::design::Mode::In => Mode::Out,
                crate::design::Mode::Out => Mode::In,
            },
            Origin::Source => match streamlet_mode {
                crate::design::Mode::In => Mode::In,
                crate::design::Mode::Out => Mode::Out,
            },
        }
    }
}

impl Portify for Interface {
    fn user(&self, name: impl Into<String>, type_name: impl Into<String>) -> Vec<Port> {
        let n: String = name.into();
        let tn: String = type_name.into();

        let mut result = Vec::new();

        // Split the LogicalStreamType up into discrete, simple streams.
        for (path, simple_stream) in self.typ().split().streams() {
            if let Some(typ) = simple_stream.user(cat!(tn.clone(), path)) {
                result.push(Port::new(cat!(n.clone(), path), self.mode().into(), typ));
            }
        }

        result
    }

    fn canonical(&self, prefix: impl Into<String>) -> Vec<Port> {
        let signals = self.typ().canonical(prefix.into());
        signals
            .iter()
            .map(|s| {
                Port::new(
                    s.identifier(),
                    s.origin().mode_for(self.mode()),
                    s.width().into(),
                )
            })
            .collect()
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

impl Componentify for Streamlet {
    fn user(&self, _: impl Into<String>) -> Option<Component> {
        Some(Component {
            identifier: self.identifier().to_string(),
            parameters: vec![],
            ports: {
                // TODO(johanpel): rust lords make me a flatmap
                let mut all_ports = Vec::new();
                self.interfaces().into_iter().for_each(|interface| {
                    all_ports.extend(interface.user(
                        interface.identifier(),
                        cat!(self.identifier().to_string(), interface.identifier()),
                    ));
                });
                all_ports
            },
        })
    }

    fn canonical(&self, _: impl Into<String>) -> Component {
        Component {
            identifier: self.identifier().to_string(),
            parameters: vec![],
            ports: {
                let mut all_ports = Vec::new();
                self.interfaces().into_iter().for_each(|interface| {
                    all_ports.extend(interface.canonical(interface.identifier()));
                });
                all_ports
            },
        }
    }
}

impl From<crate::design::Library> for Library {
    fn from(l: crate::design::Library) -> Self {
        Library {
            identifier: l.identifier().to_string(),
            components: l
                .streamlets()
                .into_iter()
                .map(|s| s.user(s.identifier()))
                .filter_map(|s| s)
                .collect(),
        }
    }
}

impl From<crate::design::Project> for Project {
    fn from(p: crate::design::Project) -> Self {
        Project {
            identifier: p.identifier().to_string(),
            libraries: p.libraries().into_iter().map(|l| l.into()).collect(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::design::{Interface, Streamlet};
    use crate::generator::common::test::records;
    use crate::generator::vhdl::Declare;
    use crate::logical::tests::{elements, streams};
    use crate::{Name, Positive, UniquelyNamedBuilder};

    #[test]
    fn test_cat() {
        assert_eq!(cat!("ok"), "ok");
        assert_eq!(cat!("ok", "tydi"), "ok_tydi");
        assert_eq!(cat!("ok", "tydi", ""), "ok_tydi");
        assert_eq!(cat!("", ""), "");
    }

    mod canonical {
        use super::*;

        #[test]
        fn logical_to_common_prim() {
            let typ = elements::prim(8).canonical("test");
            assert_eq!(
                typ,
                vec![Signal::vec(
                    "test".to_string(),
                    Origin::Source,
                    Positive::new(8).unwrap()
                )]
            )
        }

        #[test]
        fn logical_to_common_groups() {
            let typ0 = elements::group().canonical("test");
            assert_eq!(
                typ0,
                vec![
                    Signal::vec(
                        "test_a".to_string(),
                        Origin::Source,
                        Positive::new(42).unwrap()
                    ),
                    Signal::vec(
                        "test_b".to_string(),
                        Origin::Source,
                        Positive::new(1337).unwrap()
                    )
                ]
            );

            let typ1 = elements::group_nested().canonical("test");
            assert_eq!(
                typ1,
                vec![
                    Signal::vec(
                        "test_c_a".to_string(),
                        Origin::Source,
                        Positive::new(42).unwrap()
                    ),
                    Signal::vec(
                        "test_c_b".to_string(),
                        Origin::Source,
                        Positive::new(1337).unwrap()
                    ),
                    Signal::vec(
                        "test_d_a".to_string(),
                        Origin::Source,
                        Positive::new(42).unwrap()
                    ),
                    Signal::vec(
                        "test_d_b".to_string(),
                        Origin::Source,
                        Positive::new(1337).unwrap()
                    ),
                ]
            );

            let typ2 = elements::group_of_single().canonical("test");
            assert_eq!(
                typ2,
                vec![Signal::vec(
                    "test_a".to_string(),
                    Origin::Source,
                    Positive::new(42).unwrap()
                ),]
            );
        }

        #[test]
        fn logical_to_common_streams() {
            let typ0 = streams::prim(8).canonical("test");
            dbg!(&typ0);

            let typ1 = streams::group().canonical("test");
            dbg!(&typ1);
        }

        #[test]
        fn interface_to_port() {
            let if0 =
                Interface::try_new("test", crate::design::Mode::In, streams::prim(8)).unwrap();
            dbg!(if0.canonical("test"));
            let if1 =
                Interface::try_new("test", crate::design::Mode::Out, streams::group()).unwrap();
            dbg!(if1.canonical("test"));
        }
    }

    mod user {
        use super::*;
        use crate::generator::common::Field;

        #[test]
        fn logical_to_common_prim() {
            let typ: Type = elements::prim(8).user("test").unwrap();
            assert_eq!(typ, records::prim(8));
        }

        #[test]
        fn logical_to_common_groups() {
            let typ0: Type = elements::group().user("test").unwrap();
            assert_eq!(typ0, records::rec("test"));

            let typ1: Type = elements::group_nested().user("test").unwrap();
            assert_eq!(typ1, records::rec_nested("test"));

            let typ2: Type = elements::group_of_single().user("test").unwrap();
            assert_eq!(typ2, records::rec_of_single("test"));
        }

        #[test]
        fn logical_to_common_streams() {
            let typ0: Type = streams::prim(8).user("test").unwrap();
            assert_eq!(
                typ0,
                Type::Record(Record {
                    identifier: "test_type".to_string(),
                    fields: vec![
                        Field::new("valid", Type::Bit, false),
                        Field::new("ready", Type::Bit, true),
                        Field::new("data", Type::bitvec(8), false)
                    ]
                })
            );

            let typ1: Type = streams::group().user("test").unwrap();
            assert_eq!(
                typ1,
                Type::Record(Record {
                    identifier: "test_type".to_string(),
                    fields: vec![
                        Field::new(
                            "a",
                            Type::Record(Record {
                                identifier: "test_a_type".to_string(),
                                fields: vec![
                                    Field::new("valid", Type::Bit, false),
                                    Field::new("ready", Type::Bit, true),
                                    Field::new("data", Type::bitvec(42), false)
                                ]
                            }),
                            false
                        ),
                        Field::new(
                            "b",
                            Type::Record(Record {
                                identifier: "test_b_type".to_string(),
                                fields: vec![
                                    Field::new("valid", Type::Bit, false),
                                    Field::new("ready", Type::Bit, true),
                                    Field::new("data", Type::bitvec(1337), false)
                                ]
                            }),
                            false
                        )
                    ]
                })
            );
        }

        #[test]
        fn interface_to_port() {
            let if0 =
                Interface::try_new("test", crate::design::Mode::In, streams::prim(8)).unwrap();
            dbg!(if0.user("test", "test"));
            let if1 =
                Interface::try_new("test", crate::design::Mode::Out, streams::group()).unwrap();
            dbg!(if1.user("test", "test"));
        }
    }

    #[test]
    pub(crate) fn simple_streamlet() -> Result<()> {
        let streamlet = Streamlet::from_builder(
            Name::try_new("test")?,
            UniquelyNamedBuilder::new().with_items(vec![
                Interface::try_new("x", crate::design::Mode::In, streams::prim(8))?,
                Interface::try_new("y", crate::design::Mode::Out, streams::group())?,
            ]),
        )?;
        // TODO(johanpel): write actual test
        let common_streamlet = streamlet.user("test").unwrap();
        let pkg = Library {
            identifier: "boomer".to_string(),
            components: vec![common_streamlet],
        };
        println!("{}", pkg.declare()?);
        Ok(())
    }

    #[test]
    pub(crate) fn nested_streams_streamlet() -> Result<()> {
        let streamlet = Streamlet::from_builder(
            Name::try_new("test")?,
            UniquelyNamedBuilder::new().with_items(vec![
                Interface::try_new("x", crate::design::Mode::In, streams::prim(8))?,
                Interface::try_new("y", crate::design::Mode::Out, streams::nested())?,
            ]),
        )?;
        // TODO(johanpel): write actual test
        let common_streamlet = streamlet.user("test").unwrap();
        let pkg = Library {
            identifier: "testing".to_string(),
            components: vec![common_streamlet],
        };
        println!("{}", pkg.declare()?);
        Ok(())
    }

    //
    // #[test]
    // fn interface_to_canonical() -> Result<()> {
    //     let interface = Interface::try_new(
    //         "x",
    //         crate::design::Mode::Out,
    //         crate::logical::tests::streams::single_element(),
    //     )?;
    //     let ports: Vec<Port> = interface.into();
    //     // TODO(johanpel): write actual test
    //     dbg!(ports);
    //     Ok(())
    // }

    // #[test]
    // fn physical_low_complexity() -> Result<()> {
    //     let phys = PhysicalStream::try_new(vec![("a", 4), ("b", 8)], 2, 0, 2, vec![])?;
    //     let common_type = phys.synthesize("test");
    //     // TODO(johanpel): write actual test
    //     println!("{}", common_type.declare().unwrap());
    //     Ok(())
    // }

    // #[test]
    // fn physical_high_complexity() -> Result<()> {
    //     let phys = PhysicalStream::try_new(
    //         vec![("a", 4), ("b", 8)],
    //         4,
    //         3,
    //         8,
    //         vec![("muh", 3), ("foo", 4)],
    //     )?;
    //
    //     let common_type = phys.synthesize("test");
    //
    //     let mut comp = Component {
    //         identifier: "MyComp".to_string(),
    //         parameters: vec![],
    //         ports: vec![Port::new("x", Mode::In, common_type)],
    //     };
    //
    //     comp.flatten_types();
    //     // TODO(johanpel): write actual test
    //     println!("{}", comp.declare().unwrap());
    //     Ok(())
    // }
}
