//! Common hardware representation for back-ends.
//!
//! The goal of this module is to define a common representation of the hardware structure to be
//! generated, before selecting a specific back-end to generate some language-specific sources.
//!
//! # Examples:
//!
//! ```
//! use tydi::generator::{
//!     chisel::ChiselBackEnd, vhdl::VHDLBackEnd,
//!     common::Project,
//!     GenerateProject
//! };
//!
//! let tmpdir = tempfile::tempdir()?;
//! let path = tmpdir.path().join("output");
//!
//! let proj = Project {
//!     identifier: "MyProj".to_string(),
//!     libraries: vec![ /*stuff*/]
//! };
//!
//! let vhdl = VHDLBackEnd::default();
//! //let chisel = ChiselBackEnd::default();
//!
//! vhdl.generate(&proj, &path.join("vhdl"));
//! //chisel.generate(&proj, &path.join("chisel"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::NonNegative;

/// Inner struct for `Type::Array`
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    /// VHDL identifier for this array type.
    pub identifier: String,
    /// The size of the array.
    pub size: usize,
    /// The type of the array elements.
    pub typ: Box<Type>,
}

/// A field for a `Record`.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Name of the field.
    pub name: String,
    /// Type of the field.
    pub typ: Type,
    /// Whether the direction of this field should be reversed
    /// w.r.t. the other fields in the record for bulk connections.
    pub reversed: bool,
}

impl Field {
    /// Construct a new record field.
    pub fn new(name: impl Into<String>, typ: Type, reversed: bool) -> Field {
        Field {
            name: name.into(),
            typ,
            reversed,
        }
    }
}

/// Inner struct for `Type::Record`
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// VHDL identifier for this record type.
    pub identifier: String,
    /// The fields of the record.
    pub fields: Vec<Field>,
}

impl Record {
    /// Construct a new record.
    pub fn new(name: impl Into<String>, fields: Vec<Field>) -> Record {
        Record {
            identifier: name.into(),
            fields,
        }
    }

    /// Construct a new record without any fields.
    pub fn new_empty(name: impl Into<String>) -> Record {
        Record {
            identifier: name.into(),
            fields: vec![],
        }
    }

    /// Construct a new record with a valid and ready bit.
    pub fn new_empty_stream(name: impl Into<String>) -> Record {
        Record {
            identifier: name.into(),
            fields: vec![
                Field::new("valid", Type::Bit, false),
                Field::new("ready", Type::Bit, true),
            ],
        }
    }

    /// Add a field to the record.
    pub fn insert_field(&mut self, name: impl Into<String>, typ: Type, reversed: bool) {
        self.fields.push(Field::new(name, typ, reversed));
    }
}

/// Hardware types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A single bit.
    Bit,
    /// A vector of bits.
    BitVec {
        /// The width of the vector.
        width: NonNegative,
    },
    /// A statically-sized array.
    Array(Array),
    /// A record.
    Record(Record),
}

/// Bundle of names and types. Useful to represent flattened types.
pub type TypeBundle = Vec<(Vec<String>, Type, bool)>;

impl Type {
    /// Construct a bit vector type.
    pub fn bitvec(width: NonNegative) -> Type {
        Type::BitVec { width }
    }

    /// Construct a record type.
    pub fn record(name: impl Into<String>, fields: Vec<Field>) -> Type {
        Type::Record(Record::new(name.into(), fields))
    }

    /// Flatten a type to a non-nested type bundle.
    pub fn flatten(&self, prefix: Vec<String>, reversed: bool) -> TypeBundle {
        let mut result: TypeBundle = vec![];
        match self {
            Type::Record(rec) => rec.fields.iter().for_each(|field| {
                let mut new_prefix = prefix.clone();
                new_prefix.push(field.name.clone());
                result.extend(field.typ.flatten(new_prefix, field.reversed))
            }),
            _ => result.push((prefix, self.clone(), reversed)),
        }
        result
    }
}

/// A parameter for components.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
}

/// Modes for ports.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    /// Input.
    In,
    /// Output.
    Out,
    /// Bidirectional.
    Inout,
    /// None.
    None,
}

/// A port.
#[derive(Debug, Clone)]
pub struct Port {
    /// Port identifier.
    pub identifier: String,
    /// Port mode.
    pub mode: Mode,
    /// Port type.
    pub typ: Type,
}

impl Port {
    pub fn new(name: impl Into<String>, mode: Mode, typ: Type) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
        }
    }
}

/// A component.
#[derive(Debug, Clone)]
pub struct Component {
    /// Component identifier.
    pub identifier: String,
    /// The parameters of the component..
    pub parameters: Vec<Parameter>,
    /// The ports of the component.
    pub ports: Vec<Port>,
}

impl Component {
    pub fn flatten_types(&mut self) {
        let mut new_ports: Vec<Port> = Vec::new();
        self.ports.iter().for_each(|port| {
            let bundle = port
                .typ
                .flatten(vec![port.identifier.clone()], port.mode == Mode::Out);
            for tup in bundle {
                new_ports.push(Port::new(
                    tup.0.join("_"),
                    if tup.2 { Mode::Out } else { Mode::In },
                    tup.1,
                ));
            }
        });
        self.ports = new_ports;
    }
}

/// A library of components and types.
#[derive(Debug)]
pub struct Library {
    /// The identifier.
    pub identifier: String,
    /// The components declared within the library.
    pub components: Vec<Component>,
}

/// A project with libraries
// TODO(johanpel): consider renaming this, because project might imply some EDA tool-specific
//                 project
#[derive(Debug)]
pub struct Project {
    /// The name of the project.
    pub identifier: String,
    /// The libraries contained within the projects.
    pub libraries: Vec<Library>,
}

#[cfg(test)]
pub(crate) mod test {

    use super::*;
    use crate::cat;

    pub(crate) mod records {

        use super::*;

        pub(crate) fn prim(bits: u32) -> Type {
            Type::bitvec(bits)
        }

        pub(crate) fn rec(name: impl Into<String>) -> Type {
            Type::record(
                cat!(name.into(), "type"),
                vec![
                    Field::new("a", Type::bitvec(42), false),
                    Field::new("b", Type::bitvec(1337), false),
                ],
            )
        }

        pub(crate) fn rec_of_single(name: impl Into<String>) -> Type {
            Type::record(
                cat!(name.into(), "type"),
                vec![Field::new("a", Type::bitvec(42), false)],
            )
        }

        pub(crate) fn rec_nested(name: impl Into<String>) -> Type {
            let n: String = name.into();
            Type::record(
                cat!(n, "type"),
                vec![
                    Field::new("c", rec(cat!(n, "c")), false),
                    Field::new("d", rec(cat!(n, "d")), false),
                ],
            )
        }
    }

    // Some structs from this mod to be used in tests:
    pub fn test_rec() -> Type {
        Type::record(
            "rec",
            vec![
                Field::new("a", Type::Bit, false),
                Field::new("b", Type::bitvec(4), true),
            ],
        )
    }

    pub fn test_rec_nested() -> Type {
        Type::record(
            cat!("rec", "nested"),
            vec![
                Field::new("a", Type::Bit, false),
                Field::new("b", test_rec(), false),
            ],
        )
    }

    pub fn test_comp() -> Component {
        Component {
            identifier: "test_comp".to_string(),
            parameters: vec![],
            ports: vec![
                Port::new("a", Mode::In, test_rec()),
                Port::new("b", Mode::Out, test_rec_nested()),
            ],
        }
    }

    pub fn test_lib() -> Library {
        Library {
            identifier: "lib".to_string(),
            components: vec![test_comp()],
        }
    }

    pub fn test_proj() -> Project {
        Project {
            identifier: "proj".to_string(),
            libraries: vec![test_lib()],
        }
    }

    #[test]
    fn test_flatten_rec() {
        let flat = test_rec().flatten(vec![], false);
        assert_eq!(flat[0].0, vec!["a".to_string()]);
        assert_eq!(flat[0].1, Type::Bit);
        assert_eq!(flat[0].2, false);
        assert_eq!(flat[1].0, vec!["b".to_string()]);
        assert_eq!(flat[1].1, Type::bitvec(4));
        assert_eq!(flat[1].2, true);
    }

    #[test]
    fn test_flatten_rec_nested() {
        let flat = test_rec_nested().flatten(vec![], false);
        assert_eq!(flat[0].0[0], "a".to_string());
        assert_eq!(flat[0].1, Type::Bit);
        assert_eq!(flat[0].2, false);
        assert_eq!(flat[1].0[0], "b".to_string());
        assert_eq!(flat[1].0[1], "a".to_string());
        assert_eq!(flat[1].1, Type::Bit);
        assert_eq!(flat[1].2, false);
        assert_eq!(flat[2].0[0], "b".to_string());
        assert_eq!(flat[2].0[1], "b".to_string());
        assert_eq!(flat[2].1, Type::bitvec(4));
        assert_eq!(flat[2].2, true);
    }
}
