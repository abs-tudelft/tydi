//! Implementations of VHDL traits for common representation.

use std::borrow::Borrow;
use std::collections::HashMap;

use crate::error::Error::BackEndError;
use crate::generator::common::{Component, Mode, Package, Port, Record, Type};
use crate::generator::vhdl::{Analyze, Declare, DeclareType, DeclareUsings, Split, VHDLIdentifier};
use crate::traits::Identify;
use crate::{cat, Document, Result};

impl VHDLIdentifier for Mode {
    fn vhdl_identifier(&self) -> Result<String> {
        match self {
            Mode::In => Ok("in".to_string()),
            Mode::Out => Ok("out".to_string()),
        }
    }
}

fn declare_rec(rec: &Record) -> Result<String> {
    let mut children = String::new();
    let mut this = format!("type {} is record\n", cat!(rec.vhdl_identifier()?));

    for field in rec.fields() {
        // Declare all nested record types first.
        if let Type::Record(nested) = field.typ() {
            children.push_str(nested.declare(false)?.clone().as_str());
            children.push_str("\n\n");
        };

        // Declare this record.
        this.push_str(
            format!(
                "  {} : {};\n",
                field.identifier(),
                field.typ().vhdl_identifier()?
            )
            .as_str(),
        );
    }
    this.push_str("end record;");
    if !children.is_empty() {
        Ok(format!("{}{}", children, this))
    } else {
        Ok(this)
    }
}

impl DeclareType for Record {
    fn declare(&self, is_root_type: bool) -> Result<String> {
        let mut result = String::new();
        if self.has_reversed() {
            let (dn, up) = self.split();
            let suffixed_dn = dn
                .unwrap()
                .append_name_nested(if is_root_type { "dn" } else { "" });
            let suffixed_up = up
                .unwrap()
                .append_name_nested(if is_root_type { "up" } else { "" });
            result.push_str(declare_rec(&suffixed_dn)?.as_str());
            result.push_str("\n\n");
            result.push_str(declare_rec(&suffixed_up)?.as_str());
        } else {
            result.push_str(declare_rec(self)?.as_str());
        }
        Ok(result)
    }
}

impl DeclareType for Type {
    fn declare(&self, is_root_type: bool) -> Result<String> {
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
            Type::Record(rec) => rec.declare(is_root_type),
            Type::Union(_) => todo!(),
            Type::Array(_) => todo!(),
        }
    }
}

impl VHDLIdentifier for Type {
    fn vhdl_identifier(&self) -> Result<String> {
        // Records and arrays use type definitions.
        // Any other types are used directly.
        match self {
            Type::Record(rec) => rec.vhdl_identifier(),
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
                let mut result: Vec<Type> = vec![self.clone()];
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
        let mut result = String::new();
        if let Some(doc) = self.doc() {
            result.push_str("--");
            result.push_str(doc.replace("\n", "\n    --").as_str());
            result.push_str("\n    ");
        }
        result.push_str(
            format!(
                "{} : {} {}",
                self.identifier(),
                self.mode().vhdl_identifier()?,
                self.typ().vhdl_identifier()?
            )
            .as_str(),
        );
        Ok(result)
    }
}

impl VHDLIdentifier for Port {
    fn vhdl_identifier(&self) -> Result<String> {
        Ok(self.identifier().to_string())
    }
}

impl Declare for Component {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.doc() {
            result.push_str("--");
            result.push_str(doc.replace("\n", "\n--").as_str());
            result.push('\n');
        }
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
                    result.push('\n');
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
            if let Type::Record(_) = p.typ() {
                result.push(p.typ())
            }
        }
        result
    }
}

impl Declare for Package {
    fn declare(&self) -> Result<String> {
        let mut result = String::new();
        result.push_str(self.declare_usings()?.as_str());
        result.push_str(format!("package {} is\n\n", self.identifier).as_str());

        // Whatever generated the common representation is responsible to not to use the same
        // identifiers for different types.
        // Use a set to remember which type identifiers we've already used, so we don't declare
        // them twice, and produce an error otherwise.
        let mut type_ids = HashMap::<String, Type>::new();
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

impl DeclareUsings for Package {
    fn declare_usings(&self) -> Result<String> {
        let mut usings = String::new();
        let mut types = self
            .components
            .iter()
            .flat_map(|x| x.ports().iter().map(|p| p.typ()));
        fn uses_std_logic(t: &Type) -> bool {
            match t {
                Type::Bit => true,
                Type::BitVec { width: _ } => true,
                Type::Record(rec) => rec.fields().any(|field| uses_std_logic(field.typ())),
                _ => false,
            }
        }

        // Very basic for now, but could become useful if other kinds of types are supported
        if types.any(|x| uses_std_logic(&x)) {
            usings.push_str("library ieee;\nuse ieee.std_logic_1164.all;\n\n");
        }

        Ok(usings)
    }
}

#[cfg(test)]
mod test {
    use crate::generator::common::test::*;

    use super::*;

    #[test]
    fn mode_decl() {
        let m0 = Mode::In;
        let m1 = Mode::Out;
        assert_eq!(m0.vhdl_identifier().unwrap(), "in");
        assert_eq!(m1.vhdl_identifier().unwrap(), "out");
    }

    #[test]
    fn prim_type_decl() {
        let t0 = Type::Bit;
        assert_eq!(t0.declare(true).unwrap(), "std_logic");

        let t1 = Type::BitVec { width: 8 };
        assert_eq!(t1.declare(true).unwrap(), "std_logic_vector(7 downto 0)");
    }

    #[test]
    fn record_type_decl() {
        let t0 = records::rec_rev("rec");
        assert_eq!(
            t0.declare(true).unwrap(),
            concat!(
                "type rec_dn_type is record\n",
                "  c : std_logic_vector(41 downto 0);\n",
                "end record;\n",
                "\n",
                "type rec_up_type is record\n",
                "  d : std_logic_vector(1336 downto 0);\n",
                "end record;"
            )
        );

        let t1 = records::rec_rev_nested("rec");
        assert_eq!(
            t1.declare(true).unwrap(),
            concat!(
                "type rec_a_dn_type is record\n",
                "  c : std_logic_vector(41 downto 0);\n",
                "  d : std_logic_vector(1336 downto 0);\n",
                "end record;\n",
                "\n",
                "type rec_b_dn_type is record\n",
                "  c : std_logic_vector(41 downto 0);\n",
                "end record;\n",
                "\n",
                "type rec_dn_type is record\n",
                "  a : rec_a_dn_type;\n",
                "  b : rec_b_dn_type;\n",
                "end record;\n",
                "\n",
                "type rec_b_up_type is record\n",
                "  d : std_logic_vector(1336 downto 0);\n",
                "end record;\n",
                "\n",
                "type rec_up_type is record\n",
                "  b : rec_b_up_type;\n",
                "end record;"
            )
        );
    }

    #[test]
    fn port_decl() {
        let p = Port::new("test", Mode::In, Type::BitVec { width: 10 });
        assert_eq!(
            "test : in std_logic_vector(9 downto 0)",
            p.declare().unwrap()
        );
    }

    #[test]
    fn comp_decl() {
        let c = test_comp().with_doc(" My awesome\n Component".to_string());
        assert_eq!(
            c.declare().unwrap(),
            concat!(
                "-- My awesome
-- Component
component test_comp
  port(
    a_dn : in a_dn_type;
    a_up : out a_up_type;
    b_dn : out b_dn_type;
    b_up : in b_up_type
  );
end component;"
            )
        );
    }

    #[test]
    fn package_decl() {
        let p = Package {
            identifier: "test".to_string(),
            components: vec![test_comp()],
        };
        assert_eq!(
            p.declare().unwrap(),
            "library ieee;
use ieee.std_logic_1164.all;

package test is

type a_dn_type is record
  c : std_logic_vector(41 downto 0);
end record;

type a_up_type is record
  d : std_logic_vector(1336 downto 0);
end record;

type b_a_dn_type is record
  c : std_logic_vector(41 downto 0);
  d : std_logic_vector(1336 downto 0);
end record;

type b_b_dn_type is record
  c : std_logic_vector(41 downto 0);
end record;

type b_dn_type is record
  a : b_a_dn_type;
  b : b_b_dn_type;
end record;

type b_b_up_type is record
  d : std_logic_vector(1336 downto 0);
end record;

type b_up_type is record
  b : b_b_up_type;
end record;

component test_comp
  port(
    a_dn : in a_dn_type;
    a_up : out a_up_type;
    b_dn : out b_dn_type;
    b_up : in b_up_type
  );
end component;

end test;"
        )
    }
}
