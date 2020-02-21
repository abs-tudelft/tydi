//! Implementations of Chisel traits for common representation.

use std::collections::HashMap;

use crate::generator::common::{Component, Library, Mode, Port, Type};
use crate::generator::chisel::{Analyze, Declare, Identify, ChiselError, ChiselResult};

impl Identify for Mode {
    fn identify(&self) -> ChiselResult {
        match self {
            Mode::In => Ok("Input".to_string()),
            Mode::Out => Ok("Output".to_string()),
            _ => Err(ChiselError::NotSynthesizable),
        }
    }
}

impl Declare for Type {
    fn declare(&self) -> ChiselResult {
        match self {
            Type::Bit => Ok("Bool()".to_string()),
            Type::BitVec { width } => {
                let actual_width = if *width == 0 { 1 } else { *width };
                Ok(format!(
                    "UInt({}.W)",
                    actual_width
                ))
            }
            Type::Record(rec) => {
                let mut result = format!("class {} extends Bundle {{ \n", rec.identifier);
                //Handle nested bundles (new operator)
                for field in &rec.fields {
                    match &field.typ {
                        Type::Record(_rec)=>   result.push_str(format!("  val {} = new {};\n", field.name, field.typ.identify()?).as_str()),
                        _   => result.push_str(format!("  val {} = {};\n", field.name, field.typ.identify()?).as_str()),
                    }
                }
                result.push_str("}");
                Ok(result)
            }
            Type::Array(arr) => Ok(format!(
                "Vec ({}, {})",
                arr.size - 1,
                arr.typ.declare()?
            )),
        }
    }
}

impl Identify for Type {
    fn identify(&self) -> ChiselResult {
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
    fn declare(&self) -> ChiselResult {
        Ok(format!(
            "val {} = {}({})",
            self.identifier,
            self.mode.identify()?,
            //Handle custom bundle
            match &self.typ {
                    Type::Record(_rec) => "new ".to_string(),
                    _   =>  "".to_string(),
                } + &self.typ.identify()?
        ))
    }
}

impl Identify for Port {
    fn identify(&self) -> ChiselResult {
        Ok(self.identifier.to_string())
    }
}

impl Analyze for Port {
    fn list_record_types(&self) -> Vec<Type> {
        self.typ.list_record_types()
    }
}

impl Declare for Component {
    fn declare(&self) -> ChiselResult {
        let mut result = String::new();
        result.push_str(format!("class {} extends Module {{\n", self.identifier).as_str());
        if !self.ports.is_empty() {
            let mut ports = self.ports.iter().peekable();
            result.push_str(" val io =  IO(new Bundle(\n");
            while let Some(p) = ports.next() {
                result.push_str("    ");
                result.push_str(p.declare()?.to_string().as_str());

                if ports.peek().is_some() {
                    result.push_str(";\n");
                } else {
                    result.push_str("\n");
                }
            }
            result.push_str("  ))\n")
        }
        result.push_str("\n\n// User code\n\n");
        result.push_str("}");
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
    fn declare(&self) -> ChiselResult {
        // Whatever generated the common representation is responsible to not to use the same
        // identifiers for different types.
        // Use a set to remember which type identifiers we've already used, so we don't declare
        // them twice.
        let mut type_ids = HashMap::<String, Type>::new();
        let mut result = String::new();
        result.push_str(format!("package {};\n\nimport chisel3._ \n\n", self.identifier).as_str());
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
                            return Err(ChiselError::TypeNameConflict);
                        }
                    }
                }
            }
            result.push_str(format!("{}\n\n", c.declare()?).as_str());
        }
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
        println!("{}", m0.identify().unwrap());
        println!("{}", m1.identify().unwrap());
    }

    #[test]
    fn test_type_decl() {
        let t0 = Type::Bit;
        let t1 = Type::BitVec { width: 8 };
        let t2 = test_rec();
        let t3 = test_rec_nested();
        println!("{}", t0.declare().unwrap());
        println!("{}", t1.declare().unwrap());
        println!("{}", t2.declare().unwrap());
        println!("{}", t3.declare().unwrap());
    }

    #[test]
    fn test_port_decl() {
        let p = Port::new("test", Mode::In, Type::BitVec { width: 10 });
        println!("{}", p.declare().unwrap());
    }

    #[test]
    fn test_comp_decl() {
        let c = test_comp();
        println!("{}", c.declare().unwrap());
    }

    #[test]
    fn test_package_decl() {
        let p = Library {
            identifier: "test".to_string(),
            components: vec![test_comp()],
        };
        println!("{}", p.declare().unwrap());
    }
}