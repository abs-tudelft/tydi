//! Generator methods and implementations for Tydi types.
//!
//! The generator module is enabled by the `generator` feature flag.

use crate::{
    generator::common::{Component, Field, Mode, Port, Record, Type},
    physical::{BitField, Dir},
    LogicalStream, PhysicalStream, Streamlet,
};

pub mod common;
pub mod vhdl;

fn log2ceil(v: usize) -> usize {
    (v as f64).log2().ceil() as usize
}

impl From<BitField> for Type {
    fn from(b: BitField) -> Self {
        Type::Record(Record {
            identifier: b.identifier.unwrap_or_else(|| "anon".to_string()),
            fields: b
                .children
                .into_iter()
                .map(|child| {
                    Field::new(
                        child
                            .identifier
                            .clone()
                            .unwrap_or_else(|| "anon".to_string()),
                        child.into(),
                    )
                })
                .collect(),
        })
    }
}

impl From<Dir> for Mode {
    fn from(d: Dir) -> Self {
        match d {
            Dir::Downstream => Mode::Out,
            Dir::Upstream => Mode::In,
        }
    }
}

impl From<PhysicalStream> for Type {
    fn from(s: PhysicalStream) -> Self {
        // Creates a type from a physical `Stream` according to Tidy spec.
        let name: String = s.identifier.join("_");
        let mut result = Record::empty(name);

        // Valid/Ready handshake ports
        result.add_field("valid", Type::Bit);
        result.add_field_rev("ready", Type::Bit);

        // Condition E > 0
        if s.elements_per_transfer > 0 {
            // Data elements
            result.add_field(
                "data",
                Type::bitvec(s.elements_per_transfer * s.fields.width_recursive()),
            );
        }

        // Transfer metadata

        // Start index (stai) Condition C >= 6 and N > 1
        if (s.complexity.major() >= 6) && (s.elements_per_transfer > 1) {
            result.add_field(
                "stai",
                Type::BitVec {
                    width: log2ceil(s.elements_per_transfer),
                },
            );
        }

        // End index (endi) condition (C >=5 or D >= 1) and (N > 1)
        if ((s.complexity.major() >= 5) || (s.dimensionality >= 1)) && (s.elements_per_transfer > 1)
        {
            result.add_field(
                "endi",
                Type::BitVec {
                    width: log2ceil(s.elements_per_transfer),
                },
            );
        }

        // Strobe (strb) condition  C >= 7 or D >= 1
        if s.complexity.major() >= 7 || s.dimensionality >= 1 {
            result.add_field(
                "strb",
                Type::BitVec {
                    width: s.elements_per_transfer,
                },
            );
        }

        // Dimensional data

        // Condition D >= 1
        if s.dimensionality >= 1 {
            result.add_field(
                "last",
                Type::BitVec {
                    width: s.dimensionality,
                },
            );
        }

        // User data

        // Condition U > 0
        if s.user_bits > 0 {
            result.add_field("user", Type::Bit);
        }

        // Return a common type.
        Type::Record(result)
    }
}

fn append_io(logical_streams: &[LogicalStream], to: &mut Vec<Port>, mode: Mode) {
    for logical_stream in logical_streams {
        let physical_streams: Vec<PhysicalStream> = logical_stream.as_phys(vec![]);
        for physical_stream in physical_streams.into_iter() {
            to.push(Port {
                identifier: physical_stream.identifier.join("_"),
                mode,
                typ: physical_stream.into(),
            });
        }
    }
}

impl From<Streamlet> for Component {
    fn from(s: Streamlet) -> Self {
        let mut result = Component {
            identifier: s.identifier,
            parameters: vec![],
            ports: vec![],
        };
        // Obtain the physicals streams of each logical stream and append them
        // with the appropriate mode.
        append_io(&s.inputs, &mut result.ports, Mode::In);
        append_io(&s.outputs, &mut result.ports, Mode::Out);
        result
    }
}

#[cfg(test)]
mod test {
    use crate::{
        generator::{
            common::{Component, Library, Type},
            vhdl::Declare,
        },
        parser::streamlet::streamlet_interface_definition,
        physical::{BitField, Complexity, Dir},
        PhysicalStream, Streamlet,
    };

    fn test_stream() -> PhysicalStream {
        PhysicalStream {
            identifier: vec!["test".to_string()],
            fields: BitField {
                identifier: None,
                width: 0,
                children: vec![
                    BitField::new(Some("a".to_string()), 1),
                    BitField::new(Some("b".to_string()), 2),
                ],
            },
            elements_per_transfer: 1,
            dimensionality: 0,
            dir: Dir::Downstream,
            complexity: Complexity::new_major(8),
            user_bits: 0,
        }
    }

    #[test]
    fn test_from_stream_to_type() {
        let p = test_stream();
        let typ: Type = p.into();
        match typ {
            Type::Record(rec) => {
                assert_eq!(rec.identifier, "test".to_string());
                assert_eq!(rec.fields[0].name, "valid".to_string());
                assert_eq!(rec.fields[0].typ, Type::Bit);
                assert_eq!(rec.fields[0].reversed, false);
                assert_eq!(rec.fields[1].name, "ready".to_string());
                assert_eq!(rec.fields[1].typ, Type::Bit);
                assert_eq!(rec.fields[1].reversed, true);
                assert_eq!(rec.fields[2].name, "data".to_string());
                assert_eq!(rec.fields[2].typ, Type::bitvec(3));
                assert_eq!(rec.fields[2].reversed, false);
                assert_eq!(rec.fields[3].name, "strb".to_string());
                assert_eq!(rec.fields[3].typ, Type::bitvec(1));
                assert_eq!(rec.fields[3].reversed, false);
            }
            _ => panic!("expected record, got something else."),
        };
    }

    #[test]
    fn test_from_stream_to_type_with_ept_dim() {
        let mut p = test_stream();
        p.dimensionality = 2;
        p.elements_per_transfer = 3;
        let typ: Type = p.into();
        dbg!(&typ);
        match typ {
            Type::Record(rec) => {
                assert_eq!(rec.identifier, "test".to_string());
                assert_eq!(rec.fields[0].name, "valid".to_string());
                assert_eq!(rec.fields[0].typ, Type::Bit);
                assert_eq!(rec.fields[0].reversed, false);
                assert_eq!(rec.fields[1].name, "ready".to_string());
                assert_eq!(rec.fields[1].typ, Type::Bit);
                assert_eq!(rec.fields[1].reversed, true);
                assert_eq!(rec.fields[2].name, "data".to_string());
                assert_eq!(rec.fields[2].typ, Type::bitvec(3 * 3));
                assert_eq!(rec.fields[2].reversed, false);
                assert_eq!(rec.fields[3].name, "stai".to_string());
                assert_eq!(rec.fields[3].typ, Type::bitvec(2));
                assert_eq!(rec.fields[3].reversed, false);
                assert_eq!(rec.fields[4].name, "endi".to_string());
                assert_eq!(rec.fields[4].typ, Type::bitvec(2));
                assert_eq!(rec.fields[4].reversed, false);
                assert_eq!(rec.fields[5].name, "strb".to_string());
                assert_eq!(rec.fields[5].typ, Type::bitvec(3));
                assert_eq!(rec.fields[5].reversed, false);
                assert_eq!(rec.fields[6].name, "last".to_string());
                assert_eq!(rec.fields[6].typ, Type::bitvec(2));
                assert_eq!(rec.fields[6].reversed, false);
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_from_stream_to_type_with_ept_dim_lowest_c() {
        let mut p = test_stream();
        p.dimensionality = 2;
        p.elements_per_transfer = 3;
        p.complexity = Complexity::default();
        let typ: Type = p.into();
        dbg!(&typ);
        match typ {
            Type::Record(rec) => {
                assert_eq!(rec.identifier, "test".to_string());
                assert_eq!(rec.fields[0].name, "valid".to_string());
                assert_eq!(rec.fields[0].typ, Type::Bit);
                assert_eq!(rec.fields[0].reversed, false);
                assert_eq!(rec.fields[1].name, "ready".to_string());
                assert_eq!(rec.fields[1].typ, Type::Bit);
                assert_eq!(rec.fields[1].reversed, true);
                assert_eq!(rec.fields[2].name, "data".to_string());
                assert_eq!(rec.fields[2].typ, Type::bitvec(3 * 3));
                assert_eq!(rec.fields[2].reversed, false);
                assert_eq!(rec.fields[3].name, "endi".to_string());
                assert_eq!(rec.fields[3].typ, Type::bitvec(2));
                assert_eq!(rec.fields[3].reversed, false);
                assert_eq!(rec.fields[4].name, "strb".to_string());
                assert_eq!(rec.fields[4].typ, Type::bitvec(3));
                assert_eq!(rec.fields[4].reversed, false);
                assert_eq!(rec.fields[5].name, "last".to_string());
                assert_eq!(rec.fields[5].typ, Type::bitvec(2));
                assert_eq!(rec.fields[5].reversed, false);
            }
            _ => panic!(),
        };
    }

    #[test]
    fn test_simple_streamlet() {
        let streamlet = streamlet_interface_definition(
            r#"MuhStreamlet

a: Bits<1>
b: Rev<Dim<Bits<1>>>

c: Group<x: Bits<3>, y: Bits<4>>
d: Bits<4>"#,
        );
        dbg!(&streamlet);
        let streamlet: Streamlet = streamlet.unwrap().1;
        let mut comp: Component = streamlet.into();
        comp.flatten_types();
        let pkg = Library {
            identifier: "Tydi".to_string(),
            components: vec![comp],
        };
        let code: String = pkg.declare();
        println!("{}", code);
    }
}
