use crate::generator::vhdl::{ListUsings, Usings};

use super::ObjectDeclaration;

impl ListUsings for ObjectDeclaration {
    fn list_usings(&self) -> crate::Result<crate::generator::vhdl::Usings> {
        match self.default() {
            Some(ak) => ak.list_usings(),
            None => Ok(Usings::new_empty()),
        }
    }
}
