use std::fmt;

use crate::generator::common::{Component, Type};
use crate::stdlib::common::architecture::assignment::CanAssign;
use crate::{Error, Identify, Name, Result};

use super::assignment::{AssignConstraint, Assignment, RangeConstraint};
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
                self.can_assign(&default, None);
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
    /// Optional constraint - when assigning to or from the alias, this is used to determine the fields it represents
    constraint: Option<AssignConstraint>,
}

impl<'a> AliasDeclaration<'a> {
    pub fn new(
        object: &'a ObjectDeclaration,
        identifier: Name,
        constraint: AssignConstraint,
    ) -> Result<AliasDeclaration<'a>> {
        AliasDeclaration::from_object(object, identifier).with_constraint(constraint)
    }

    pub fn from_object(object: &'a ObjectDeclaration, identifier: Name) -> AliasDeclaration<'a> {
        AliasDeclaration {
            identifier,
            object,
            constraint: None,
        }
    }

    pub fn with_constraint(mut self, constraint: AssignConstraint) -> Result<Self> {
        // TODO: Verify object supports constraint
        match self.object().typ() {
            ObjectType::Bit => Err(Error::InvalidTarget(
                "Cannot alias a bit object with a constraint".to_string(),
            )),
            ObjectType::Array(array) => {
                if let AssignConstraint::Range(range_constraint) = constraint {
                    if range_constraint.high() <= array.high()
                        && range_constraint.low() >= array.low()
                    {
                        self.constraint = Some(constraint);
                        Ok(self)
                    } else {
                        Err(Error::InvalidArgument(format!("Cannot alias an array with range constraint {}, array has high: {}, low: {}", range_constraint, array.high(), array.low())))
                    }
                } else {
                    Err(Error::InvalidTarget(
                        "Cannot alias an array with a named field".to_string(),
                    ))
                }
            }
            ObjectType::Record(_) => todo!(),
        }
        // self.constraint = Some(constraint);
        // Ok(self)
    }

    /// Returns the actual object this is aliasing
    pub fn object(&self) -> &'a ObjectDeclaration {
        self.object
    }

    /// Returns the optional fixed range constraint of this alias
    pub fn constraint(&self) -> &Option<AssignConstraint> {
        &self.constraint
    }

    /// Returns the alias's identifier
    pub fn identifier(&self) -> &Name {
        &self.identifier
    }
}
