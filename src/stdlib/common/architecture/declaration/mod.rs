use std::fmt;

use crate::generator::common::{Component, Type};
use crate::stdlib::common::architecture::assignment::CanAssign;
use crate::{Error, Identify, Name, Result};

use super::assignment::{Assignment, FieldSelection, RangeConstraint};
use super::object::ObjectType;

// Declarations may typically be any of the following: type, subtype, signal, constant, file, alias, component, attribute, function, procedure, configuration specification. (per: https://www.ics.uci.edu/~jmoorkan/vhdlref/architec.html)
// Per: https://insights.sigasi.com/tech/vhdl2008.ebnf/#block_declarative_item
//     subprogram_declaration
// | subprogram_body
// | subprogram_instantiation_declaration
// | package_declaration
// | package_body
// | package_instantiation_declaration
// | type_declaration
// | subtype_declaration
// | constant_declaration
// | signal_declaration
// | shared_variable_declaration
// | file_declaration
// | alias_declaration
// | component_declaration
// | attribute_declaration
// | attribute_specification
// | configuration_specification
// | disconnection_specification
// | use_clause
// | group_template_declaration
// | group_declaration
// | PSL_Property_Declaration
// | PSL_Sequence_Declaration
// | PSL_Clock_Declaration
/// Architecture declaration.
#[derive(Debug, Clone)]
pub enum ArchitectureDeclaration<'a> {
    /// Type declarations within the architecture
    Type(Type),
    SubType(String), // TODO: Do we want subtypes, or should these just be (part of) types?

    Procedure(String), // TODO: Procedure
    Function(String),  // TODO: Function
    /// Object declaration, covering signals, variables, constants and ports*
    ///
    /// *Ports cannot be declared within the architecture itself, but can be used in the statement part,
    /// as such, the ports of the entity implemented are treated as inferred declarations.
    Object(ObjectDeclaration),
    /// Alias for an object declaration, with optional range constraint
    Alias(AliasDeclaration<'a>),
    /// Component declarations within the architecture
    Component(Component),
    Custom(String), // TODO: Custom (templates?)
}

/// The kind of object declared (signal, variable, constant, port)
#[derive(Debug, Clone)]
pub enum ObjectKind {
    Signal,
    Variable,
    Constant,
    Port,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjectKind::Signal => write!(f, "Signal"),
            ObjectKind::Variable => write!(f, "Variable"),
            ObjectKind::Constant => write!(f, "Constant"),
            ObjectKind::Port => write!(f, "Port"),
        }
    }
}

/// Struct describing the identifier of the object, its type, its kind, and a potential default value
#[derive(Debug, Clone)]
pub struct ObjectDeclaration {
    /// Name of the signal
    identifier: Name,
    /// (Sub-)Type of the object
    typ: ObjectType,
    /// Default value assigned to the object (required for constants, cannot be used for ports)
    default: Option<Assignment>,
    /// The kind of object
    kind: ObjectKind,
}

impl ObjectDeclaration {
    pub fn signal(
        identifier: Name,
        typ: ObjectType,
        default: Option<Assignment>,
    ) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default,
            kind: ObjectKind::Signal,
        }
    }

    pub fn variable(
        identifier: Name,
        typ: ObjectType,
        default: Option<Assignment>,
    ) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default,
            kind: ObjectKind::Variable,
        }
    }

    pub fn constant(identifier: Name, typ: ObjectType, value: Assignment) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default: Some(value),
            kind: ObjectKind::Constant,
        }
    }

    pub fn port(identifier: Name, typ: ObjectType) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default: None,
            kind: ObjectKind::Port,
        }
    }

    pub fn set_default(mut self, default: Assignment) -> Result<()> {
        match self.kind() {
            ObjectKind::Signal | ObjectKind::Variable => {
                // self.can_assign(&default, None);
                self.default = Some(default);
                Ok(())
            }
            ObjectKind::Constant | ObjectKind::Port => Err(Error::InvalidTarget(format!(
                "Default cannot be assigned to {} object",
                self.kind()
            ))),
        }
    }

    pub fn kind(&self) -> &ObjectKind {
        &self.kind
    }

    pub fn typ(&self) -> &ObjectType {
        &self.typ
    }

    pub fn identifier(&self) -> &Name {
        &self.identifier
    }

    pub fn default(&self) -> &Option<Assignment> {
        &self.default
    }
}

/// Aliases an existing object, with optional field constraint
#[derive(Debug, Clone)]
pub struct AliasDeclaration<'a> {
    identifier: Name,
    /// Reference to an existing object declaration
    object: &'a ObjectDeclaration,
    /// Optional field selection(s) - when assigning to or from the alias, this is used to determine the fields it represents
    field_selection: Vec<FieldSelection>,
}

impl<'a> AliasDeclaration<'a> {
    pub fn new(
        object: &'a ObjectDeclaration,
        identifier: Name,
        fields: Vec<FieldSelection>,
    ) -> Result<AliasDeclaration<'a>> {
        AliasDeclaration::from_object(object, identifier).with_selection(fields)
    }

    pub fn from_object(object: &'a ObjectDeclaration, identifier: Name) -> AliasDeclaration<'a> {
        AliasDeclaration {
            identifier,
            object,
            field_selection: vec![],
        }
    }

    /// Apply one or more field selections to the alias
    pub fn with_selection(mut self, fields: Vec<FieldSelection>) -> Result<Self> {
        let mut object = self.object().typ().clone();
        for field in self.field_selection() {
            object = object.get_field(field)?;
        }
        for field in fields {
            object = object.get_field(&field)?;
            self.field_selection.push(field)
        }

        Ok(self)
    }

    /// Returns the actual object this is aliasing
    pub fn object(&self) -> &'a ObjectDeclaration {
        self.object
    }

    /// Returns the optional fixed range constraint of this alias
    pub fn field_selection(&self) -> &Vec<FieldSelection> {
        &self.field_selection
    }

    /// Returns the alias's identifier
    pub fn identifier(&self) -> &Name {
        &self.identifier
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use indexmap::IndexMap;

    use super::*;

    fn test_bit_signal() -> Result<ObjectDeclaration> {
        Ok(ObjectDeclaration::signal(
            Name::try_from("test_signal")?,
            ObjectType::Bit,
            None,
        ))
    }

    fn test_complex_signal() -> Result<ObjectDeclaration> {
        let mut record: IndexMap<Name, ObjectType> = IndexMap::new();
        record.insert(
            Name::try_from("a")?,
            ObjectType::array(10, -4, ObjectType::Bit)?,
        );
        Ok(ObjectDeclaration::signal(
            Name::try_from("test_signal")?,
            ObjectType::Record(record),
            None,
        ))
    }

    #[test]
    fn alias_verification_success() -> Result<()> {
        AliasDeclaration::from_object(&test_bit_signal()?, Name::try_from("test_signal_alias")?);
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![FieldSelection::name("a")?])?;
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![
            FieldSelection::name("a")?,
            FieldSelection::downto(10, -4)?,
        ])?;
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![FieldSelection::name("a")?])?
        .with_selection(vec![FieldSelection::downto(10, -4)?])?;
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![
            FieldSelection::name("a")?,
            FieldSelection::downto(4, -1)?,
        ])?;
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![
            FieldSelection::name("a")?,
            FieldSelection::to(-4, 10)?,
        ])?;
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![FieldSelection::name("a")?, FieldSelection::index(10)])?;
        AliasDeclaration::from_object(
            &test_complex_signal()?,
            Name::try_from("test_signal_alias")?,
        )
        .with_selection(vec![FieldSelection::name("a")?, FieldSelection::index(-4)])?;
        Ok(())
    }

    #[test]
    fn alias_verification_error() -> Result<()> {
        is_invalid_target(
            AliasDeclaration::from_object(
                &test_bit_signal()?,
                Name::try_from("test_signal_alias")?,
            )
            .with_selection(vec![FieldSelection::name("a")?]),
        )?;
        is_invalid_target(
            AliasDeclaration::from_object(
                &test_bit_signal()?,
                Name::try_from("test_signal_alias")?,
            )
            .with_selection(vec![FieldSelection::index(1)]),
        )?;
        is_invalid_target(
            AliasDeclaration::from_object(
                &test_complex_signal()?,
                Name::try_from("test_signal_alias")?,
            )
            .with_selection(vec![FieldSelection::index(1)]),
        )?;
        is_invalid_argument(
            AliasDeclaration::from_object(
                &test_complex_signal()?,
                Name::try_from("test_signal_alias")?,
            )
            .with_selection(vec![FieldSelection::name("b")?]),
        )?;
        is_invalid_target(
            AliasDeclaration::from_object(
                &test_complex_signal()?,
                Name::try_from("test_signal_alias")?,
            )
            .with_selection(vec![FieldSelection::name("a")?, FieldSelection::name("a")?]),
        )?;
        is_invalid_argument(
            AliasDeclaration::from_object(
                &test_complex_signal()?,
                Name::try_from("test_signal_alias")?,
            )
            .with_selection(vec![
                FieldSelection::name("a")?,
                FieldSelection::downto(11, -4)?,
            ]),
        )?;
        Ok(())
    }

    fn is_invalid_target<T>(result: Result<T>) -> Result<()> {
        match result {
            Err(Error::InvalidTarget(_)) => Ok(()),
            _ => Err(Error::UnknownError),
        }
    }

    fn is_invalid_argument<T>(result: Result<T>) -> Result<()> {
        match result {
            Err(Error::InvalidArgument(_)) => Ok(()),
            _ => Err(Error::UnknownError),
        }
    }
}
