//! Common hardware representation.
//!
//! The goal of this module is to define some common constructs seen in structural hardware
//! generation that back-ends may or may not use.

use indexmap::IndexMap;

use crate::traits::Identify;
use crate::{cat, Document, Name};
use crate::{NonNegative, Reversed};

pub mod convert;

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
        let mut result = Record::new_empty(cat!(self.identifier, p));
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

/// Inner struct for `Type::Union`
#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    identifier: String,
    types: IndexMap<Name, Type>
}

impl Identify for Union {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

/// Inner struct for `Type::Array`
#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    identifier: String,
    typ: Box<Type>,
    width: NonNegative,
}

impl Identify for Array {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
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
    /// A record.
    Record(Record),
    /// Unions are implemented as a record containing the data and tag, and an enumeration of the types which can be contained in it
    Union(Union),
    /// An array of any type
    Array(Array),
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

    /// Construct an array type.
    pub fn array(name: impl Into<String>, typ: Type, width: u32) -> Type {
        Type::Array(Array {
            identifier: name.into(),
            typ: Box::new(typ),
            width,
        })
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
    /// Port documentation.
    doc: Option<String>,
}

impl Port {
    /// Create a new port.
    pub fn new(name: impl Into<String>, mode: Mode, typ: Type) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
            doc: None,
        }
    }

    /// Create a new port with documentation.
    pub fn new_documented(
        name: impl Into<String>,
        mode: Mode,
        typ: Type,
        doc: Option<String>,
    ) -> Port {
        Port {
            identifier: name.into(),
            mode,
            typ,
            doc,
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

    /// Return this port with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this port.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

impl Identify for Port {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl Document for Port {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
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
    /// Documentation.
    doc: Option<String>,
}

impl Identify for Component {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl Document for Component {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Component {
    /// Create a new component.
    pub fn new(
        identifier: impl Into<String>,
        parameters: Vec<Parameter>,
        ports: Vec<Port>,
        doc: Option<String>,
    ) -> Component {
        Component {
            identifier: identifier.into(),
            parameters,
            ports,
            doc,
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

    /// Return this component with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this component.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

/// A library of components and types.
#[derive(Debug)]
pub struct Package {
    /// The identifier.
    pub identifier: String,
    /// The components declared within the library.66
    pub components: Vec<Component>,
}

/// A project with libraries
#[derive(Debug)]
pub struct Project {
    /// The name of the project.
    pub identifier: String,
    /// The libraries contained within the projects.
    pub libraries: Vec<Package>,
}

#[cfg(test)]
pub(crate) mod test {
    use crate::cat;

    use super::*;

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
                    Field::new("a", rec(cat!(n, "a")), false),
                    Field::new("b", rec_rev(cat!(n, "b")), false),
                ],
            )
        }

        pub(crate) fn rec_nested(name: impl Into<String>) -> Type {
            let n: String = name.into();
            Type::record(
                n.clone(),
                vec![
                    Field::new("a", rec(cat!(n, "a")), false),
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
                Port::new_documented("a", Mode::In, records::rec_rev("a"), None),
                Port::new_documented("b", Mode::Out, records::rec_rev_nested("b"), None),
            ],
            doc: None,
        }
    }

    #[test]
    fn flatten_rec() {
        let flat = records::rec("test").flatten(vec![], false);
        assert_eq!(flat[0].0, vec!["c".to_string()]);
        assert_eq!(flat[0].1, Type::bitvec(42));
        assert!(!flat[0].2);
        assert_eq!(flat[1].0, vec!["d".to_string()]);
        assert_eq!(flat[1].1, Type::bitvec(1337));
        assert!(!flat[1].2);
    }

    #[test]
    fn flatten_rec_nested() {
        let flat = records::rec_nested("test").flatten(vec![], false);
        dbg!(&flat);
        assert_eq!(flat[0].0[0], "a".to_string());
        assert_eq!(flat[0].0[1], "c".to_string());
        assert_eq!(flat[0].1, Type::bitvec(42));
        assert!(!flat[0].2);
        assert_eq!(flat[1].0[0], "a".to_string());
        assert_eq!(flat[1].0[1], "d".to_string());
        assert_eq!(flat[1].1, Type::bitvec(1337));
        assert!(!flat[1].2);
        assert_eq!(flat[2].0[0], "b".to_string());
        assert_eq!(flat[2].0[1], "c".to_string());
        assert_eq!(flat[2].1, Type::bitvec(42));
        assert!(!flat[2].2);
        assert_eq!(flat[3].0[0], "b".to_string());
        assert_eq!(flat[3].0[1], "d".to_string());
        assert_eq!(flat[3].1, Type::bitvec(1337));
        assert!(!flat[3].2);
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
