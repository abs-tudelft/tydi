//! Implementations of VHDL traits for common representation.

use crate::error::Error::BackEndError;
use crate::generator::common::{Component, Library, Mode, Port, Record, Type};
use crate::generator::vhdl::{Analyze, Declare, DeclareType, Split, VHDLIdentifier};
use crate::traits::Identify;
use crate::{cat, Result};
use std::collections::HashMap;

impl VHDLIdentifier for Mode {
    fn vhdl_identifier(&self) -> Result<String> {
        match self {
            Mode::In => Ok("in".to_string()),
            Mode::Out => Ok("out".to_string()),
        }
    }
}

fn declare_rec(rec: &Record) -> Result<String> {
    let mut result = format!("record {}\n", rec.vhdl_identifier()?);
    for field in rec.fields() {
        result.push_str(
            format!(
                "  {} : {};\n",
                field.identifier(),
                field.typ().vhdl_identifier()?
            )
            .as_str(),
        );
    }
    result.push_str("end record;");
    Ok(result)
}

impl DeclareType for Type {
    fn declare(&self, is_root: bool) -> Result<String> {
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
                let mut result = String::new();
                if rec.has_reversed() {
                    let (dn, up) = rec.split();
                    let suffixed_dn =
                        dn.unwrap()
                            .append_name_nested(if is_root { "dn" } else { "" });
                    let suffixed_up =
                        up.unwrap()
                            .append_name_nested(if is_root { "up" } else { "" });
                    result.push_str(declare_rec(&suffixed_dn)?.as_str());
                    result.push_str("\n\n");
                    result.push_str(declare_rec(&suffixed_up)?.as_str());
                } else {
                    result.push_str(declare_rec(&rec)?.as_str());
                }
                Ok(result)
            }
            Type::Array(arr) => Ok(format!(
                "array ({} to {}) of {}",
                0,
                arr.size - 1,
                arr.typ.declare(is_root)?
            )),
        }
    }
}

impl VHDLIdentifier for Type {
    fn vhdl_identifier(&self) -> Result<String> {
        // Records and arrays use type definitions.
        // Any other types are used directly.
        match self {
            Type::Record(rec) => rec.vhdl_identifier(),
            Type::Array(arr) => Ok(arr.identifier().to_string()),
            _ => self.declare(true),
        }
    }
}

impl VHDLIdentifier for Record {
    fn vhdl_identifier(&self) -> Result<String> {
        Ok(cat!(self.identifier().to_string(), "type"))
    }
}

impl Analyze for Type {
    fn list_record_types(&self) -> Vec<Type> {
        match self {
            // Only record can have nested records.
            Type::Record(rec) => {
                let mut result: Vec<Type> = vec![];
                result.push(self.clone());
                for f in rec.fields().into_iter() {
                    let children = f.typ().list_record_types();
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
            self.identifier(),
            self.mode().vhdl_identifier()?,
            self.typ().vhdl_identifier()?
        ))
    }
}

impl VHDLIdentifier for Port {
    fn vhdl_identifier(&self) -> Result<String> {
        Ok(self.identifier().to_string())
    }
}

impl Analyze for Port {
    fn list_record_types(&self) -> Vec<Type> {
        self.typ().list_record_types()
    }
}

impl Declare for Component {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        result.push_str(format!("component {}\n", self.identifier()).as_str());
        if !self.ports().is_empty() {
            let mut ports = self.ports().iter().peekable();
            result.push_str("  port(\n");
            while let Some(p) = ports.next() {
                result.push_str("    ");
                // If the port type has reversed fields, we need to split it up because VHDL.
                if p.has_reversed() {
                    let (dn, up) = p.split();
                    match dn {
                        None => unreachable!(),
                        Some(dn_port) => {
                            result.push_str(dn_port.declare()?.as_str());
                            result.push_str(";\n");
                        }
                    };
                    match up {
                        None => unreachable!(),
                        Some(up_port) => {
                            result.push_str("    ");
                            result.push_str(up_port.declare()?.as_str());
                        }
                    };
                } else {
                    result.push_str(p.declare()?.as_str());
                }

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
        for p in self.ports().iter() {
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
                match type_ids.get(&r.vhdl_identifier()?) {
                    None => {
                        type_ids.insert(r.vhdl_identifier()?, r.clone());
                        result.push_str(format!("{}\n\n", r.declare(true)?).as_str());
                    }
                    Some(already_defined_type) => {
                        if r != already_defined_type {
                            return Err(BackEndError(format!(
                                "Type name conflict: {}",
                                already_defined_type
                                    .vhdl_identifier()
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
        assert_eq!(m0.vhdl_identifier().unwrap(), "in");
        assert_eq!(m1.vhdl_identifier().unwrap(), "out");
    }

    #[test]
    fn test_type_decl() {
        let t0 = Type::Bit;
        let t1 = Type::BitVec { width: 8 };
        let t2 = test_rec();
        let t3 = test_rec_nested();
        assert_eq!(t0.declare(true).unwrap(), "std_logic");
        assert_eq!(t1.declare(true).unwrap(), "std_logic_vector(7 downto 0)");
        assert_eq!(
            t2.declare(true).unwrap(),
            concat!(
                "record rec_dn_type\n",
                "  c : std_logic;\n",
                "end record;\n",
                "\n",
                "record rec_up_type\n",
                "  d : std_logic_vector(3 downto 0);\n",
                "end record;"
            )
        );
        assert_eq!(
            t3.declare(true).unwrap(),
            concat!(
                "record rec_nested_dn_type\n",
                "  a : std_logic;\n",
                "  b : rec_dn_type;\n",
                "end record;\n",
                "\n",
                "record rec_nested_up_type\n",
                "  b : rec_up_type;\n",
                "end record;"
            )
        );
    }

    #[test]
    fn test_port_decl() {
        let p = Port::new("test", Mode::In, Type::BitVec { width: 10 });
        assert_eq!(
            "test : in std_logic_vector(9 downto 0)",
            p.declare().unwrap()
        );
    }

    #[test]
    fn test_comp_decl() {
        let c = test_comp();
        assert_eq!(
            c.declare().unwrap(),
            concat!(
                "component test_comp\n",
                "  port(\n",
                "    a_dn : in rec_dn_type;\n",
                "    a_up : out rec_up_type;\n",
                "    b_dn : out rec_nested_dn_type;\n",
                "    b_up : in rec_nested_up_type\n",
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
                "package test is\n",
                "\n",
                "record rec_dn_type\n",
                "  c : std_logic;\n",
                "end record;\n",
                "\n",
                "record rec_up_type\n",
                "  d : std_logic_vector(3 downto 0);\n",
                "end record;\n",
                "\n",
                "record rec_nested_dn_type\n",
                "  a : std_logic;\n",
                "  b : rec_dn_type;\n",
                "end record;\n",
                "\n",
                "record rec_nested_up_type\n",
                "  b : rec_up_type;\n",
                "end record;\n",
                "\n",
                "component test_comp\n",
                "  port(\n",
                "    a_dn : in rec_dn_type;\n",
                "    a_up : out rec_up_type;\n",
                "    b_dn : out rec_nested_dn_type;\n",
                "    b_up : in rec_nested_up_type\n",
                "  );\n",
                "end component;\n",
                "\n",
                "end test;",
            )
        )
    }
}
