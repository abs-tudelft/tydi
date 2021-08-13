use crate::generator::common::{Component, Type};
use crate::Name;

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
pub enum ArchitectureDeclaration {
    /// Type declarations within the architecture
    Type(Type),
    SubType(String), // TODO: Do we want subtypes, or should these just be (part of) types?
    Alias(String),   // TODO: Alias
    Procedure(String), // TODO: Procedure
    Function(String), // TODO: Function
    Signal(String), // TODO: Signal (not quite the same as physical::Signal, should support Types and default values)
    Constant(String), // TODO: Constant
    /// Component declarations within the architecture
    Component(Component),
    Custom(String), // TODO: Custom (templates?)
}

pub struct Signal {
    /// Name of the signal
    identifier: Name,
    /// (Sub-)Type of the signal
    typ: Type,
}
