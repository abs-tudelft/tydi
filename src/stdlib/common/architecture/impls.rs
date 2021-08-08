use crate::generator::vhdl::ListUsings;

use super::*;

impl ListUsings for Architecture {
    fn list_usings(&self) -> Result<Usings> {
        Ok(self.usings.clone())
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
        let architecture = Architecture::new_work(package, Name::try_new("test").unwrap()).unwrap();
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
}
