use crate::{stdlib::common::architecture::ArchitectureDeclare, Error, Result};

use super::{PortMapping, Statement};

impl ArchitectureDeclare for PortMapping {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        let mut result = pre.to_string();
        result.push_str(&format!(
            "{}: {} port map(\n",
            self.label(),
            self.component_name()
        ));
        let mut port_maps = vec![];
        for (port, _) in self.ports() {
            if let Some(port_assign) = self.mappings().get(port) {
                port_maps.push(port_assign.declare(&format!("{}  ", pre), "")?);
            } else {
                return Err(Error::BackEndError(format!(
                    "Error while declaring port mapping, port {} is not assigned",
                    port
                )));
            }
        }
        result.push_str(&port_maps.join(",\n"));
        result.push_str(&format!("\n{}){}", pre, post));
        Ok(result)
    }
}

impl ArchitectureDeclare for Statement {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        match self {
            Statement::Assignment(assignment) => assignment.declare(pre, post),
            Statement::PortMapping(portmapping) => portmapping.declare(pre, post),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use indexmap::IndexMap;

    use super::*;
    use crate::{
        generator::{
            common::test::{
                records::{rec_rev, rec_rev_nested},
                test_comp,
            },
            vhdl::Split,
        },
        stdlib::common::architecture::{
            assignment::{bitvec::BitVecValue, AssignmentKind, StdLogicValue},
            declaration::ObjectDeclaration,
        },
    };

    #[test]
    fn test_simple_portmapping_declare() -> Result<()> {
        let (a_dn, a_up) = rec_rev("a").split();
        let a_dn_rec = ObjectDeclaration::signal("a_dn_rec", a_dn.unwrap().try_into()?, None);
        let a_up_rec = ObjectDeclaration::signal("a_up_rec", a_up.unwrap().try_into()?, None);
        let (b_dn, b_up) = rec_rev_nested("b").split();
        let b_dn_rec = ObjectDeclaration::signal("b_dn_rec", b_dn.unwrap().try_into()?, None);
        let b_up_rec = ObjectDeclaration::signal("b_up_rec", b_up.unwrap().try_into()?, None);
        let mut pm = PortMapping::from_component(&test_comp(), "some_label")?;
        let mapped = pm
            .map_port("a_dn", &a_dn_rec)?
            .map_port("a_up", &a_up_rec)?
            .map_port("b_dn", &b_dn_rec)?
            .map_port("b_up", &b_up_rec)?;
        assert_eq!(
            r#"  some_label: test_comp port map(
    a_dn => a_dn_rec,
    a_up => a_up_rec,
    b_dn => b_dn_rec,
    b_up => b_up_rec
  );
"#,
            mapped.declare("  ", ";\n")?
        );
        Ok(())
    }

    #[test]
    fn test_complex_portmapping_declare() -> Result<()> {
        let mut fields_a = IndexMap::new();
        fields_a.insert(
            "c".to_string(),
            BitVecValue::Others(StdLogicValue::Logic(true)).into(),
        );
        fields_a.insert(
            "d".to_string(),
            BitVecValue::Others(StdLogicValue::Logic(false)).into(),
        );
        let mut fields_b = IndexMap::new();
        fields_b.insert(
            "a".to_string(),
            AssignmentKind::full_record(fields_a.clone()),
        );
        fields_b.insert(
            "b".to_string(),
            AssignmentKind::full_record(fields_a.clone()),
        );
        let mut pm = PortMapping::from_component(&test_comp(), "some_label")?;
        let mapped = pm
            .map_port("a", &AssignmentKind::full_record(fields_a))?
            .map_port("b", &AssignmentKind::full_record(fields_b))?;
        assert_eq!(
            r#"some_label: test_comp port map(
  a => (
    c => (others => '1'),
    d => (others => '0')
  ),
  b => (
    a => (
      c => (others => '1'),
      d => (others => '0')
    ),
    b => (
      c => (others => '1'),
      d => (others => '0')
    )
  )
);
"#,
            pm.declare("", ";\n")?
        );
        Ok(())
    }
}
