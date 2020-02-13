//! Common hardware representation for back-ends.
//!
//! The goal of this module is to define a common representation of the hardware structure to be
//! generated, before selecting a specific back-end to generate some language-specific sources.
//!
//! # Examples:
//!
//! ```
//! use tydi::generator::vhdl::VHDLBackEnd;
//! use tydi::generator::chisel::ChiselBackEnd;
//! use tydi::generator::{common::Project, GenerateProject};
//!
//! let proj = Project {
//!     identifier: "MyProj".to_string(),
//!     libraries: vec![ /*stuff*/]
//! };
//!
//! let vhdl = VHDLBackEnd::default();
//! let chisel = ChiselBackEnd::default();
//!
//! vhdl.generate(&proj, std::Path::new("output/vhdl"));
//! chisel.generate(&proj, std::Path::new("output/chisel"));
//! ```

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
    pub fn new(name: impl Into<String>, typ: Type) -> Field {
        Field {
            name: name.into(),
            typ,
            reversed: false,
        }
    }

    /// Construct a new record field.
    pub fn new_rev(name: impl Into<String>, typ: Type) -> Field {
        Field {
            name: name.into(),
            typ,
            reversed: true,
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
    pub fn empty(name: impl Into<String>) -> Record {
        Record {
            identifier: name.into(),
            fields: vec![],
        }
    }

    /// Add a field to the record.
    pub fn add_field(&mut self, name: impl Into<String>, typ: Type) {
        self.fields.push(Field::new(name, typ));
    }

    /// Add a reversed field to the record, i.e. in bulk connections it will flow in
    /// opposite direction w.r.t. the other record fields.
    pub fn add_field_rev(&mut self, name: impl Into<String>, typ: Type) {
        self.fields.push(Field::new_rev(name, typ));
    }
}

/// VHDL types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A single bit.
    Bit,
    /// A vector of bits.
    BitVec {
        /// The width of the vector.
        width: usize,
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
    pub fn bitvec(width: usize) -> Type {
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
                .flatten(vec![port.identifier.clone()], port.mode == Mode::In);
            for tup in bundle {
                new_ports.push(Port::new(
                    tup.0.join("_"),
                    if tup.2 { Mode::In } else { Mode::Out },
                    tup.1,
                ));
            }
        });
        self.ports = new_ports;
    }
}

/// A library of components and types.
pub struct Library {
    /// The identifier.
    pub identifier: String,
    /// The components declared within the library.
    pub components: Vec<Component>,
}

/// A project with libraries
// TODO(johanpel): consider renaming this, because project might imply some EDA tool-specific
//                 project
pub struct Project {
    /// The name of the project.
    pub identifier: String,
    /// The libraries contained within the projects.
    pub libraries: Vec<Library>,
}

#[cfg(test)]
pub mod test {
    use super::*;

    // Some structs from this mod to be used in tests:
    pub fn test_rec() -> Type {
        Type::record(
            "rec",
            vec![
                Field::new("a", Type::Bit),
                Field::new_rev("b", Type::bitvec(4)),
            ],
        )
    }

    pub fn test_rec_nested() -> Type {
        Type::record(
            "rec_nested",
            vec![Field::new("a", Type::Bit), Field::new("b", test_rec())],
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
        dbg!(&flat);
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
