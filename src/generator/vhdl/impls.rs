//! Implementations of VHDL traits for common representation.

use std::collections::HashMap;

use crate::error::Error::BackEndError;
use crate::generator::common::{Component, Library, Mode, Port, Type};
use crate::generator::vhdl::{Analyze, Declare, Identify};
use crate::Result;

impl Identify for Mode {
    fn identify(&self) -> Result<String> {
        match self {
            Mode::In => Ok("in".to_string()),
            Mode::Out => Ok("out".to_string()),
            Mode::Inout => Ok("inout".to_string()),
            _ => Err(BackEndError("Mode not synthesizable.".to_string())),
        }
    }
}

impl Declare for Type {
    fn declare(&self) -> Result<String> {
        match self {
            Type::Bit => Ok("std_logic".to_string()),
            Type::BitVec { width } => {
                let actual_width = if *width == 0 { 1 } else { *width };
                Ok(format!(
                    "std_logic_vector({} downto {})",
                    actual_width - 1,
                    0
                ))
            }
            Type::Record(rec) => {
                let mut result = format!("record {}\n", rec.identifier);
                for field in &rec.fields {
                    result.push_str(
                        format!("  {} : {};\n", field.name, field.typ.identify()?).as_str(),
                    );
                }
                result.push_str("end record;");
                Ok(result)
            }
            Type::Array(arr) => Ok(format!(
                "array ({} to {}) of {}",
                0,
                arr.size - 1,
                arr.typ.declare()?
            )),
        }
    }
}

impl Identify for Type {
    fn identify(&self) -> Result<String> {
        // Records and arrays use type definitions.
        // Any other types are used directly.
        match self {
            Type::Record(rec) => Ok(rec.identifier.clone()),
            Type::Array(arr) => Ok(arr.identifier.clone()),
            _ => self.declare(),
        }
    }
}

impl Analyze for Type {
    fn list_record_types(&self) -> Vec<Type> {
        match self {
            // Only record can have nested records.
            Type::Record(rec) => {
                let mut result: Vec<Type> = vec![];
                result.push(self.clone());
                for f in rec.fields.iter() {
                    let children = f.typ.list_record_types();
                    result.extend(children.into_iter());
                }
                result
            }
            _ => vec![],
        }
    }
}

impl Declare for Port {
    fn declare(&self) -> Result<String> {
        Ok(format!(
            "{} : {} {}",
            self.identifier,
            self.mode.identify()?,
            self.typ.identify()?
        ))
    }
}

impl Identify for Port {
    fn identify(&self) -> Result<String> {
        Ok(self.identifier.to_string())
    }
}

impl Analyze for Port {
    fn list_record_types(&self) -> Vec<Type> {
        self.typ.list_record_types()
    }
}

impl Declare for Component {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        result.push_str(format!("component {}\n", self.identifier).as_str());
        if !self.ports.is_empty() {
            let mut ports = self.ports.iter().peekable();
            result.push_str("  port(\n");
            while let Some(p) = ports.next() {
                result.push_str("    ");
                result.push_str(p.declare()?.to_string().as_str());
                if ports.peek().is_some() {
                    result.push_str(";\n");
                } else {
                    result.push_str("\n");
                }
            }
            result.push_str("  );\n")
        }
        result.push_str("end component;");
        Ok(result)
    }
}

impl Analyze for Component {
    fn list_record_types(&self) -> Vec<Type> {
        let mut result: Vec<Type> = vec![];
        for p in &self.ports {
            let children = p.list_record_types();
            result.extend(children.into_iter());
        }
        result
    }
}

impl Declare for Library {
    fn declare(&self) -> Result<String> {
        // Whatever generated the common representation is responsible to not to use the same
        // identifiers for different types.
        // Use a set to remember which type identifiers we've already used, so we don't declare
        // them twice.
        let mut type_ids = HashMap::<String, Type>::new();
        let mut result = String::new();
        result.push_str(format!("package {} is\n\n", self.identifier).as_str());
        for c in &self.components {
            let comp_records = c.list_record_types();
            for r in comp_records.iter() {
                match type_ids.get(&r.identify()?) {
                    None => {
                        type_ids.insert(r.identify()?, r.clone());
                        result.push_str(format!("{}\n\n", r.declare()?).as_str());
                    }
                    Some(already_defined_type) => {
                        if r != already_defined_type {
                            return Err(BackEndError(format!(
                                "Type name conflict: {}",
                                already_defined_type
                                    .identify()
                                    .unwrap_or_else(|_| "".to_string())
                            )));
                        }
                    }
                }
            }
            result.push_str(format!("{}\n\n", c.declare()?).as_str());
        }
        result.push_str(format!("end {};", self.identifier).as_str());
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generator::common::test::*;

    #[test]
    fn test_mode_decl() {
        let m0 = Mode::In;
        let m1 = Mode::Out;
        assert_eq!(m0.identify().unwrap(), "in");
        assert_eq!(m1.identify().unwrap(), "out");
    }

    #[test]
    fn test_type_decl() {
        let t0 = Type::Bit;
        let t1 = Type::BitVec { width: 8 };
        let t2 = test_rec();
        let t3 = test_rec_nested();
        assert_eq!(t0.declare().unwrap(), "std_logic");
        assert_eq!(t1.declare().unwrap(), "std_logic_vector(7 downto 0)");
        assert_eq!(
            t2.declare().unwrap(),
            concat!(
                "record rec\n",
                "  a : std_logic;\n",
                "  b : std_logic_vector(3 downto 0);\n",
                "end record;"
            )
        );
        assert_eq!(
            t3.declare().unwrap(),
            concat!(
                "record rec_nested\n",
                "  a : std_logic;\n",
                "  b : rec;\n",
                "end record;"
            )
        );
    }

    #[test]
    fn test_port_decl() {
        let p = Port::new("test", Mode::In, Type::BitVec { width: 10 });
        println!("{}", p.declare().unwrap());
    }

    #[test]
    fn test_comp_decl() {
        let c = test_comp();
        assert_eq!(
            c.declare().unwrap(),
            concat!(
                "component test_comp\n",
                "  port(\n",
                "    a : in rec;\n",
                "    b : out rec_nested\n",
                "  );\n",
                "end component;"
            )
        );
    }

    #[test]
    fn test_package_decl() {
        let p = Library {
            identifier: "test".to_string(),
            components: vec![test_comp()],
        };
        assert_eq!(
            p.declare().unwrap(),
            concat!(
                "package test is\n\n",
                "record rec\n",
                "  a : std_logic;\n",
                "  b : std_logic_vector(3 downto 0);\n",
                "end record;\n",
                "\n",
                "record rec_nested\n",
                "  a : std_logic;\n",
                "  b : rec;\n",
                "end record;\n",
                "\n",
                "component test_comp\n",
                "  port(\n",
                "    a : in rec;\n",
                "    b : out rec_nested\n",
                "  );\n",
                "end component;\n",
                "\n",
                "end test;"
            )
        )
    }
}
