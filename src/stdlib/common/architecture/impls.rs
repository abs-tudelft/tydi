use crate::{Document, generator::vhdl::{Declare, DeclareUsings, ListUsings}};

use super::*;

impl ListUsings for Architecture {
    fn list_usings(&self) -> Result<Usings> {
        Ok(self.usings.clone())
    }
}


// TODO: Architecture definition
// Based on: https://insights.sigasi.com/tech/vhdl2008.ebnf/#architecture_body
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
impl Declare for Architecture {
    fn declare(&self) -> crate::Result<String> {
        let mut result = String::new();
        result.push_str(self.declare_usings()?.as_str());

        result.push_str(self.entity.declare()?.as_str());
        result.push_str("\n");

        if let Some(doc) = self.doc() {
            result.push_str("--");
            result.push_str(doc.replace("\n", "\n--").as_str());
            result.push('\n');
        }

        result.push_str(format!("architecture {} of {} is\n", self.identifier(), self.entity.identifier()).as_str());
        result.push_str("--<architecture_declarative_part>\n"); // TODO: Add declarative part
        result.push_str("begin\n");
        result.push_str("--<architecture_statement_part>\n"); // TODO: Add statement part
        result.push_str(format!("end {};\n", self.identifier()).as_str());
        Ok(result)
    }
}

impl Identify for Architecture {
    fn identifier(&self) -> &str {
        self.identifier.as_ref()
    }
}

impl Document for Architecture {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::vhdl::DeclareUsings;
    use crate::stdlib::common::architecture::tests::test_package;

    use super::*;

    #[test]
    fn architecture_declare_usings() {
        let package = test_package();
        let architecture =
            Architecture::new_default(package, Name::try_new("test").unwrap()).unwrap();
        let usings = architecture.declare_usings().unwrap();
        assert_eq!(
            usings,
            "library ieee;
use ieee.std_logic_1164.all;

library work;
use work.test.all;

"
        );
    }

    #[test]
    fn architecture_declare() {
        let package = test_package();
        let architecture =
            Architecture::new_default(package, Name::try_new("test").unwrap()).unwrap();
        let decl = architecture.declare().unwrap();
        assert_eq!(
            decl,
            "library ieee;
use ieee.std_logic_1164.all;

library work;
use work.test.all;

entity test is
  port(
    clk : in std_logic;
    rst : in std_logic;
    a_dn : in test_a_dn_type;
    a_up : out test_a_up_type;
    b_dn : out test_b_dn_type;
    b_up : in test_b_up_type
  );
end test;

architecture Behavioral of test is
--<architecture_declarative_part>
begin
--<architecture_statement_part>
end Behavioral;
"
        );
    }
}
