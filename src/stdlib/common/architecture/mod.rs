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