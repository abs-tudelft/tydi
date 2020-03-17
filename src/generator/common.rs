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

use crate::cat;
use crate::traits::Identify;
use crate::{NonNegative, Reversed};

/// Inner struct for `Type::Array`
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    /// VHDL identifier for this array type.
    identifier: String,
    /// The size of the array.
    pub size: usize,
    /// The type of the array elements.
    pub typ: Box<Type>,
}

impl Identify for Array {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

/// A field for a `Record`.
///
/// A field may be "reversed" with respect to the other fields in the record.
/// This means that when the type is used to describe a connection between an input and output port
/// of some component, this field will have its port modes swapped.
///
/// # Example:
/// ```
/// use tydi::generator::common::{Port, Mode, Record, Field, Type};
///
/// let port = Port::new("example",
///     Mode::In,
///     Type::record("rec", vec![              // Shortcut to Type::Record(Record::new(...
///         Field::new("a", Type::Bit, false), // This field will have a port Mode::In
///         Field::new("b", Type::Bit, true)   // This field will have a port Mode::Out
///     ])
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Name of the field.
    name: String,
    /// Type of the field.
    typ: Type,
    /// Whether the direction of this field should be reversed
    /// w.r.t. the other fields in the record for bulk connections.
    reversed: bool,
}

impl Identify for Field {
    fn identifier(&self) -> &str {
        self.name.as_str()
    }
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

    /// Returns the type of this field.
    pub fn typ(&self) -> &Type {
        &self.typ
    }

    /// Returns true if this field is reversed.
    pub fn is_reversed(&self) -> bool {
        self.reversed
    }
}

impl Reversed for Field {
    fn reversed(&self) -> Self {
        Field::new(self.name.clone(), self.typ.clone(), !self.reversed)
    }
}

/// Inner struct for `Type::Record`
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// VHDL identifier for this record type.
    identifier: String,
    /// The fields of the record.
    fields: Vec<Field>,
}

impl Identify for Record {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl Record {
    /// Construct a new record.
    pub fn new(name: impl Into<String>, fields: Vec<Field>) -> Record {
        // TODO(johanpel): records should be constructed through a UniquelyNamedBuilder
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

    /// Create a new field and add it to the record.
    pub fn insert_new_field(&mut self, name: impl Into<String>, typ: Type, reversed: bool) {
        self.fields.push(Field::new(name, typ, reversed));
    }

    /// Add a field to the record.
    pub fn insert(&mut self, field: Field) {
        self.fields.push(field);
    }

    /// Returns true if the record contains a field that is reversed.
    /// Does not include nested records.
    pub fn has_reversed_field(&self) -> bool {
        self.fields.iter().any(|i| i.reversed)
    }

    /// Returns true if the record contains a field that is reversed,
    /// including any nested records.
    pub fn has_reversed(&self) -> bool {
        self.fields
            .iter()
            .any(|i| i.reversed || i.typ.has_reversed())
    }

    /// Returns an iterable over the fields.
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }

    /// Returns true if record contains no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Append a string to the name of this record, and any nested records.
    pub fn append_name_nested(&self, with: impl Into<String>) -> Self {
        let p: String = with.into();
        let mut result = Record::new_empty(cat!(self.identifier, p.clone()));
        for f in self.fields() {
            result.insert(match f.typ() {
                Type::Record(r) => Field::new(
                    f.identifier(),
                    Type::Record(r.append_name_nested(p.clone())),
                    f.reversed,
                ),
                _ => f.clone(),
            });
        }
        result
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

    // Returns true if the type contains a reversed field.
    pub fn has_reversed(&self) -> bool {
        match self {
            Type::Record(rec) => rec.has_reversed(),
            _ => false,
        }
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
}

impl Reversed for Mode {
    fn reversed(&self) -> Self {
        match self {
            Mode::In => Mode::Out,
            Mode::Out => Mode::In,
        }
    }
}

/// A port.
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    /// Port identifier.
    identifier: String,
    /// Port mode.
    mode: Mode,
    /// Port type.
    typ: Type,
}

impl Port {
    /// Create a new port.
    pub fn new(name: impl Into<String>, mode: Mode, typ: Type) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
        }
    }

    /// Return the port mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Return the type of the port.
    pub fn typ(&self) -> Type {
        self.typ.clone()
    }

    /// Returns true if the port type contains reversed fields.
    pub fn has_reversed(&self) -> bool {
        self.typ.has_reversed()
    }
}

impl Identify for Port {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

/// A component.
#[derive(Debug, Clone)]
pub struct Component {
    /// Component identifier.
    identifier: String,
    /// The parameters of the component..
    parameters: Vec<Parameter>,
    /// The ports of the component.
    ports: Vec<Port>,
}

impl Identify for Component {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl Component {
    /// Create a new component.
    pub fn new(
        identifier: impl Into<String>,
        parameters: Vec<Parameter>,
        ports: Vec<Port>,
    ) -> Component {
        Component {
            identifier: identifier.into(),
            parameters,
            ports,
        }
    }

    /// Return a reference to the ports of this component.
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    /// Return a reference to the parameters of this component.
    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

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
                name.into(),
                vec![
                    Field::new("c", Type::bitvec(42), false),
                    Field::new("d", Type::bitvec(1337), false),
                ],
            )
        }

        pub(crate) fn rec_rev(name: impl Into<String>) -> Type {
            Type::record(
                name.into(),
                vec![
                    Field::new("c", Type::bitvec(42), false),
                    Field::new("d", Type::bitvec(1337), true),
                ],
            )
        }

        pub(crate) fn rec_of_single(name: impl Into<String>) -> Type {
            Type::record(name.into(), vec![Field::new("a", Type::bitvec(42), false)])
        }

        pub(crate) fn rec_rev_nested(name: impl Into<String>) -> Type {
            let n: String = name.into();
            Type::record(
                n.clone(),
                vec![
                    Field::new("a", rec(cat!(n.clone(), "a")), false),
                    Field::new("b", rec_rev(cat!(n, "b")), false),
                ],
            )
        }

        pub(crate) fn rec_nested(name: impl Into<String>) -> Type {
            let n: String = name.into();
            Type::record(
                n.clone(),
                vec![
                    Field::new("a", rec(cat!(n.clone(), "a")), false),
                    Field::new("b", rec(cat!(n, "b")), false),
                ],
            )
        }
    }

    pub fn test_comp() -> Component {
        Component {
            identifier: "test_comp".to_string(),
            parameters: vec![],
            ports: vec![
                Port::new("a", Mode::In, records::rec_rev("a")),
                Port::new("b", Mode::Out, records::rec_rev_nested("b")),
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
    fn flatten_rec() {
        let flat = records::rec("test").flatten(vec![], false);
        assert_eq!(flat[0].0, vec!["c".to_string()]);
        assert_eq!(flat[0].1, Type::bitvec(42));
        assert_eq!(flat[0].2, false);
        assert_eq!(flat[1].0, vec!["d".to_string()]);
        assert_eq!(flat[1].1, Type::bitvec(1337));
        assert_eq!(flat[1].2, false);
    }

    #[test]
    fn flatten_rec_nested() {
        let flat = records::rec_nested("test").flatten(vec![], false);
        dbg!(&flat);
        assert_eq!(flat[0].0[0], "a".to_string());
        assert_eq!(flat[0].0[1], "c".to_string());
        assert_eq!(flat[0].1, Type::bitvec(42));
        assert_eq!(flat[0].2, false);
        assert_eq!(flat[1].0[0], "a".to_string());
        assert_eq!(flat[1].0[1], "d".to_string());
        assert_eq!(flat[1].1, Type::bitvec(1337));
        assert_eq!(flat[1].2, false);
        assert_eq!(flat[2].0[0], "b".to_string());
        assert_eq!(flat[2].0[1], "c".to_string());
        assert_eq!(flat[2].1, Type::bitvec(42));
        assert_eq!(flat[2].2, false);
        assert_eq!(flat[3].0[0], "b".to_string());
        assert_eq!(flat[3].0[1], "d".to_string());
        assert_eq!(flat[3].1, Type::bitvec(1337));
        assert_eq!(flat[3].2, false);
    }

    #[test]
    fn has_reversed() {
        assert!(records::rec_rev("test").has_reversed());
        assert!(records::rec_rev_nested("test").has_reversed());
        assert!(!Type::record(
            "test",
            vec![
                Field::new("a", Type::Bit, false),
                Field::new("b", Type::Bit, false),
            ],
        )
        .has_reversed());
    }
}
