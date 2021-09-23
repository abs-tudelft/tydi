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

    use super::*;
    use crate::{
        generator::common::test::{
            records::{rec_rev, rec_rev_nested},
            test_comp,
        },
        stdlib::common::architecture::{
            assignment::AssignmentKind, declaration::ObjectDeclaration, object::ObjectType,
        },
    };

    #[test]
    fn test_simple_portmapping_declare() -> Result<()> {
        let (a_dn, a_up) = ObjectType::try_from_splittable(rec_rev("a"))?;
        let a_dn_rec = ObjectDeclaration::signal("a_dn_rec", a_dn.unwrap(), None);
        let a_up_rec = ObjectDeclaration::signal("a_up_rec", a_up.unwrap(), None);
        let (b_dn, b_up) = ObjectType::try_from_splittable(rec_rev_nested("b"))?;
        let b_dn_rec = ObjectDeclaration::signal("b_dn_rec", b_dn.unwrap(), None);
        let b_up_rec = ObjectDeclaration::signal("b_up_rec", b_up.unwrap(), None);
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
        let (a_dn, a_up) = ObjectType::try_from_splittable(rec_rev("a_other"))?;
        let a_dn_rec = ObjectDeclaration::signal("a_other_dn_rec", a_dn.unwrap(), None);
        let a_up_rec = ObjectDeclaration::signal("a_other_up_rec", a_up.unwrap(), None);
        let (b_dn, b_up) = ObjectType::try_from_splittable(rec_rev_nested("b_other"))?;
        let b_dn_rec = ObjectDeclaration::signal("b_other_dn_rec", b_dn.unwrap(), None);
        let b_up_rec = ObjectDeclaration::signal("b_other_up_rec", b_up.unwrap(), None);
        let mut pm = PortMapping::from_component(&test_comp(), "some_label")?;
        let mapped = pm
            .map_port("a_dn", &AssignmentKind::to_direct(&a_dn_rec, true)?)?
            .map_port("a_up", &AssignmentKind::to_direct(&a_up_rec, true)?)?
            .map_port("b_dn", &AssignmentKind::to_direct(&b_dn_rec, true)?)?
            .map_port("b_up", &AssignmentKind::to_direct(&b_up_rec, true)?)?;
        assert_eq!(
            r#"some_label: test_comp port map(
  a_dn => (
    c => a_other_dn_rec.c
  ),
  a_up => (
    d => a_other_up_rec.d
  ),
  b_dn => (
    a => (
      c => b_other_dn_rec.a.c,
      d => b_other_dn_rec.a.d
    ),
    b => (
      c => b_other_dn_rec.b.c
    )
  ),
  b_up => (
    b => (
      d => b_other_up_rec.b.d
    )
  )
);
"#,
            mapped.declare("", ";\n")?
        );
        Ok(())
    }
}
