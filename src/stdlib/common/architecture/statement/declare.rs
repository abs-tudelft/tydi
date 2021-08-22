use crate::{stdlib::common::architecture::ArchitectureDeclare, Result};

use super::{PortMapping, Statement};

impl ArchitectureDeclare for PortMapping {
    fn declare(&self, pre: &str, post: &str) -> Result<String> {
        let mut result = pre.to_string();
        result.push_str(&format!(
            "{}: {} port map(\n",
            self.label(),
            self.component_name()
        ));
        for (port, object) in self.ports() {
            if let Some(port_assign) = self.mappings().get(port) {
                result.push_str(&port_assign.declare(&format!("{} ", pre), ",")?);
            } else {
                result.push_str(&object.declare(&format!("{} ", pre), ",")?);
            }
        }
        result.push_str(&format!("){}\n", post));
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
