//! Source generator module.

use common::*;

use crate::{
    phys::{BitField, Dir, Stream},
    Streamlet,
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

impl From<Stream> for Type {
    fn from(s: Stream) -> Self {
        // Creates a type from a physical `Stream` according to Tidy spec.
        let name: String = s.identifier.clone().unwrap_or_else(|| "anon".to_string());
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
        if (s.complexity.num[0] >= 6) && (s.elements_per_transfer > 1) {
            result.add_field(
                "stai",
                Type::BitVec {
                    width: log2ceil(s.elements_per_transfer),
                },
            );
        }

        // End index (endi) condition (C >=5 or D >= 1) and (N > 1)
        if ((s.complexity.num[0] >= 5) || (s.dimensionality >= 1)) && (s.elements_per_transfer > 1)
        {
            result.add_field(
                "endi",
                Type::BitVec {
                    width: log2ceil(s.elements_per_transfer),
                },
            );
        }

        // Strobe (strb) condition  C >= 7 or D >= 1
        if s.complexity.num[0] >= 7 || s.dimensionality >= 1 {
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

impl From<Streamlet> for Component {
    fn from(s: Streamlet) -> Self {
        let mut result = Component {
            identifier: s.identifier,
            parameters: vec![],
            ports: vec![],
        };
        // Obtain the physicals streams of each river.
        for i in s.inputs {
            let phys_streams: Vec<Stream> = i.as_phys(i.identifier());
            for ps in phys_streams.into_iter() {
                result.ports.push(Port::new(
                    ps.identifier.clone().unwrap_or_else(|| "".to_string()),
                    Mode::In,
                    ps.into(),
                ));
            }
        }
        for o in s.outputs {
            let phys_streams: Vec<Stream> = o.as_phys(o.identifier());
            for ps in phys_streams.into_iter() {
                result.ports.push(Port::new(
                    ps.identifier.clone().unwrap_or_else(|| "".to_string()),
                    Mode::Out,
                    ps.into(),
                ));
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::generator::common::{Component, Library, Type};
    use crate::generator::vhdl::Declare;
    use crate::parser::streamlet::streamlet_interface_definition;
    use crate::phys::{BitField, Complexity, Dir, Stream};
    use crate::Streamlet;

    fn test_stream() -> Stream {
        Stream {
            identifier: Some("test".to_string()),
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
            complexity: Complexity::highest(),
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
        p.complexity = Complexity::lowest();
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

c: Group<Bits<3>, Bits<4>>
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
