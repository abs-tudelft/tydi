use crate::{Identify, Name, generator::{common::{Component, Package, Type}, vhdl::{ListUsings, Usings}}};
use crate::{Result, Error};

use super::entity::Entity;

mod impls;

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
    Procedure(String), // TODO: Procedure
    Function(String), // TODO: Function
    Signal(String), // TODO: Signal (not quite the same as physical::Signal, should Types and default values)
    Constant(String), // TODO: Constant
    /// Component declarations within the architecture
    Component(Component),
    Custom(String), // TODO: Custom (templates?)
}

// TODO: Figure this out, either make it a struct with specific contents, or a trait for something else to implement?
/// Architecture statement.
#[derive(Debug, Clone)]
pub struct ArchitectureStatement {

}

// NOTE: One of the main things to consider is probably how to handle multiple element lanes. Probably as a check on the number of lanes,
// then wrapping in a generate statement. Need to consider indexes at that point.
// This'd be easier if I simply always made it an array, even when the number of lanes is 1, but that gets real ugly, real fast.

/// Architecture declarations.
#[derive(Debug, Clone)]
pub struct ArchitectureDeclarations {

}

/// An architecture
#[derive(Debug)]
pub struct Architecture {
    /// Name of the architecture
    identifier: Name,
    /// Entity which this architecture is for
    entity: Entity,
    /// Additional usings beyond the Package and those within it
    usings: Usings,
    /// Documentation.
    doc: Option<String>,
}

impl Architecture {
    /// Create the architecture based on a component contained within a package, assuming the library (project) is "work" and the architecture's identifier is "Behavioral"
    pub fn new_default(package: Package, component_id: Name) -> Result<Architecture> {
        Architecture::new(Name::try_new("work")?, Name::try_new("Behavioral")?, package, component_id)
    }

    /// Create the architecture based on a component contained within a package, specify the library (project) in which the package is contained
    pub fn new(library_id: Name, identifier: Name, package: Package, component_id: Name) -> Result<Architecture> {
        if let Some(component) = package.components.iter().find(|x| component_id == *x.identifier()) {
            let mut usings = package.list_usings()?;
            usings.add_using(library_id, format!("{}.all", package.identifier));
            Ok(Architecture {
                identifier,
                entity: Entity::from(component.clone()),
                usings: usings,
                doc: None,
            })
        } else {
            Err(Error::InvalidArgument(format!("Identifier \"{}\" does not exist in this package", component_id)))
        }
    }

    /// Add additional usings which weren't already part of the package
    pub fn add_using(&mut self, library: Name, using: String) -> bool {
        self.usings.add_using(library, using)
    }

    /// Return this architecture with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this architecture.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::common::convert::Packify;

    use super::*;

    pub fn test_package() -> Package {
        let (_, streamlet) = crate::parser::nom::streamlet(
            "Streamlet test (a : in Stream<Bits<1>>, b : out Stream<Bits<2>, d=2>)",
        )
        .unwrap();
        let lib = crate::design::library::Library::try_new(
            Name::try_new("test").unwrap(),
            vec![],
            vec![streamlet],
        );
        let lib: crate::generator::common::Package = lib.unwrap().fancy();
        lib
    }

    #[test]
    fn new_architecture() {
        let package = test_package();
        let architecture = Architecture::new_default(package, Name::try_new("test").unwrap());

        print!("{:?}", architecture);
    }
}