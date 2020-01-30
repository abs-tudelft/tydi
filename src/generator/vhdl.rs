//! VHDL source generation support.
//!
//! This module contains helper structs, functions and traits to generate structural VHDL.

use crate::generator::common::*;

/// Generate VHDL declarations.
pub trait Declare {
    /// Generate a VHDL declaration from self.
    fn declare(&self) -> String;
}

/// Generate VHDL identifiers.
pub trait Identify {
    /// Generate a VHDL identifier from self.
    fn identify(&self) -> String;
}

/// Analyze VHDL objects.
pub trait Analyze {
    /// List all record types used.
    fn list_record_types(&self) -> Vec<Type>;
}

impl Identify for Mode {
    fn identify(&self) -> String {
        match self {
            Mode::In => "in".to_string(),
            Mode::Out => "out".to_string(),
            Mode::Inout => "inout".to_string(),
            _ => unimplemented!(),
        }
    }
}

impl Declare for Type {
    fn declare(&self) -> String {
        match self {
            Type::Bit => "std_logic".to_string(),
            Type::BitVec { width } => {
                let actual_width = if *width == 0 { 1 } else { *width };
                format!("std_logic_vector({} downto {})", actual_width - 1, 0)
            }
            Type::Record(rec) => {
                let mut result = format!("record {}\n", rec.identifier);
                for field in &rec.fields {
                    result
                        .push_str(format!("{} : {};\n", field.name, field.typ.identify()).as_str());
                }
                result.push_str("end record;");
                result
            }
            Type::Array(arr) => {
                format!("array ({} to {}) of {}", 0, arr.size - 1, arr.typ.declare())
            }
        }
    }
}

impl Identify for Type {
    fn identify(&self) -> String {
        // Records and arrays use type definitions.
        // Any other types are used directly.
        match self {
            Type::Record(rec) => rec.identifier.clone(),
            Type::Array(arr) => arr.identifier.clone(),
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
    fn declare(&self) -> String {
        format!(
            "{} : {} {}",
            self.identifier,
            self.mode.identify(),
            self.typ.identify()
        )
    }
}

impl Identify for Port {
    fn identify(&self) -> String {
        self.identifier.to_string()
    }
}

impl Analyze for Port {
    fn list_record_types(&self) -> Vec<Type> {
        self.typ.list_record_types()
    }
}

impl Declare for Component {
    fn declare(&self) -> String {
        let mut result = String::new();
        result.push_str(format!("component {}\n", self.identifier).as_str());
        if !self.ports.is_empty() {
            let mut ports = self.ports.iter().peekable();
            result.push_str("port(\n");
            while let Some(p) = ports.next() {
                result.push_str(p.declare().to_string().as_str());
                if ports.peek().is_some() {
                    result.push_str(";\n");
                }
            }
            result.push_str(");\n")
        }
        result.push_str("end component;");
        result
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
    fn declare(&self) -> String {
        let mut result = String::new();
        result.push_str(format!("package {} is\n", self.identifier).as_str());
        for c in &self.components {
            let comp_records = c.list_record_types();
            for r in comp_records.iter() {
                result.push_str(format!("{}\n\n", r.declare()).as_str());
            }
            result.push_str(format!("{}\n\n", c.declare()).as_str());
        }
        result.push_str(format!("end {};", self.identifier).as_str());
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn rec() -> Type {
        Type::record(
            "rec",
            vec![Field::new("a", Type::Bit), Field::new("b", Type::bitvec(4))],
        )
    }
    fn rec_nested() -> Type {
        Type::record(
            "rec_nested",
            vec![Field::new("a", Type::Bit), Field::new("b", rec())],
        )
    }

    fn comp() -> Component {
        Component {
            identifier: "test_comp".to_string(),
            parameters: vec![],
            ports: vec![
                Port::new("a", Mode::In, rec()),
                Port::new("b", Mode::Out, rec_nested()),
            ],
        }
    }

    #[test]
    fn test_mode_decl() {
        let m0 = Mode::In;
        let m1 = Mode::Out;
        assert_eq!(m0.identify(), "in");
        assert_eq!(m1.identify(), "out");
    }

    #[test]
    fn test_type_decl() {
        let t0 = Type::Bit;
        let t1 = Type::BitVec { width: 8 };
        let t2 = rec();
        let t3 = rec_nested();
        assert_eq!(t0.declare(), "std_logic");
        assert_eq!(t1.declare(), "std_logic_vector(7 downto 0)");
        assert_eq!(
            t2.declare(),
            concat!(
                "record rec\n",
                "a : std_logic;\n",
                "b : std_logic_vector(3 downto 0);\n",
                "end record;"
            )
        );
        assert_eq!(
            t3.declare(),
            concat!(
                "record rec_nested\n",
                "a : std_logic;\n",
                "b : rec;\n",
                "end record;"
            )
        );
    }

    #[test]
    fn test_port_decl() {
        let p = Port::new("test", Mode::In, Type::BitVec { width: 10 });
        println!("{}", p.declare());
    }

    #[test]
    fn test_comp_decl() {
        let c = comp();
        assert_eq!(
            c.declare(),
            concat!(
                "component test_comp\n",
                "port(\n",
                "a : in rec;\n",
                "b : out rec_nested);\n",
                "end component;"
            )
        );
    }

    #[test]
    fn test_package_decl() {
        let p = Library {
            identifier: "test".to_string(),
            components: vec![comp()],
        };
        println!("{}", p.declare());
    }
}
