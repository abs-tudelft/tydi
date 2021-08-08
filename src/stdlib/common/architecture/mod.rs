use crate::{Identify, Name, generator::{common::Package, vhdl::{ListUsings, Usings}}};
use crate::{Result, Error};

use super::entity::Entity;

mod impls;

// TODO: Figure this out, either make it a struct with specific contents, or a trait for something else to implement?
/// Architecture declaration.
#[derive(Debug, Clone)]
pub struct ArchitectureDeclaration {

}

// TODO: Figure this out, either make it a struct with specific contents, or a trait for something else to implement?
/// Architecture statement.
#[derive(Debug, Clone)]
pub struct ArchitectureStatement {

}

/// Architecture declarations.
#[derive(Debug, Clone)]
pub struct ArchitectureDeclarations {

}

/// An architecture
#[derive(Debug)]
pub struct Architecture {
    /// Entity which this architecture is for
    entity: Entity,
    /// Additional usings beyond the Package and those within it
    usings: Usings,
    /// Documentation.
    doc: Option<String>,
}

impl Architecture {
    /// Create the architecture based on a component contained within a package, assuming the default library (project) is "work"
    pub fn new_work(package: Package, component_id: Name) -> Result<Architecture> {
        Architecture::new(Name::try_new("work")?, package, component_id)
    }

    /// Create the architecture based on a component contained within a package, specify the library (project) in which the package is contained
    pub fn new(library_id: Name, package: Package, component_id: Name) -> Result<Architecture> {
        if let Some(component) = package.components.iter().find(|x| component_id == *x.identifier()) {
            let mut usings = package.list_usings()?;
            usings.add_using(library_id, format!("{}.all", package.identifier));
            Ok(Architecture {
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
}


// TODO: Architecture definition
// Based on: https://insights.sigasi.com/tech/vhdl2008.ebnf/
// <usings>
// architecture <identifier> of <entity_name> is
// <architecture_declarative_part>
// begin
// <architecture_statement_part>
// end architecture <identifier>;
//
// Should probably start with the declarative part (components, signals, potentially functions & procedures)
//
// Architecture overall needs:
// Usings (based on contents, what library the component came from...)
// Entity
// An identifier (Could just be "Behavioral"/"RTL")
//
// Declarative part needs:
// Components (add as needed)
// Signals (add as needed, with names and possibly defaults)
// Type declarations, based on signals
//
// Statement part can have:
// Signal assignment
// Component assignment (w/ labels) // NOTE: This is where the "drives defaults" part comes in
// Processes (which are yet another layer)
//
// Processes can have:
// Declarations (variables)
// Sequential statements
//
// Any complex logic should probably just be string templates.

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
        let architecture = Architecture::new_work(package, Name::try_new("test").unwrap());

        print!("{:?}", architecture);
    }
}