use crate::generator::common::{Component, Type};
use crate::{Identify, Name, Result};

use super::assignment::{Assignment, RangeConstraint};

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

/// Struct describing the identifier of the object, its type, its kind, and a potential default value
#[derive(Debug, Clone)]
pub struct ObjectDeclaration {
    /// Name of the signal
    identifier: Name,
    /// (Sub-)Type of the object
    typ: Type,
    /// Default value assigned to the object (required for constants, cannot be used for ports)
    default: Option<Assignment>,
    /// The kind of object
    kind: ObjectKind,
}

impl ObjectDeclaration {
    pub fn signal(identifier: Name, typ: Type, default: Option<Assignment>) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default,
            kind: ObjectKind::Signal,
        }
    }

    pub fn variable(identifier: Name, typ: Type, default: Option<Assignment>) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default,
            kind: ObjectKind::Variable,
        }
    }

    pub fn constant(identifier: Name, typ: Type, value: Assignment) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default: Some(value),
            kind: ObjectKind::Constant,
        }
    }

    pub fn port(identifier: Name, typ: Type) -> ObjectDeclaration {
        ObjectDeclaration {
            identifier,
            typ,
            default: None,
            kind: ObjectKind::Port,
        }
    }
}

/// Aliases an existing object, with optional range constraint
#[derive(Debug, Clone)]
pub struct AliasDeclaration<'a> {
    identifier: Name,
    /// Reference to an existing object declaration
    object: &'a ObjectDeclaration,
    /// Optional range constraint - when assigning to or from the alias, this is used to determine the range it represents
    range_constraint: Option<RangeConstraint>,
}

impl<'a> AliasDeclaration<'a> {
    pub fn new(
        object: &'a ObjectDeclaration,
        identifier: Name,
        range_constraint: RangeConstraint,
    ) -> Result<AliasDeclaration> {
        AliasDeclaration::from_object(object, identifier).with_range(range_constraint)
    }

    pub fn from_object(object: &'a ObjectDeclaration, identifier: Name) -> AliasDeclaration {
        AliasDeclaration {
            identifier,
            object,
            range_constraint: None,
        }
    }

    pub fn with_range(mut self, range_constraint: RangeConstraint) -> Result<Self> {
        // TODO: Verify object supports range
        self.range_constraint = Some(range_constraint);
        Ok(self)
    }

    /// Returns the actual object this is aliasing
    pub fn object(&self) -> &'a ObjectDeclaration {
        self.object
    }

    /// Returns the optional fixed range constraint of this alias
    pub fn range_constraint(&self) -> &Option<RangeConstraint> {
        &self.range_constraint
    }

    /// Returns the alias's identifier
    pub fn identifier(&self) -> Name {
        self.identifier.clone()
    }
}