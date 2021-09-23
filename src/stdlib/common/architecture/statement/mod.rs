use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{
    generator::common::Component, stdlib::common::architecture::assignment::Assign, Error,
    Identify, Result,
};

use super::{
    assignment::{AssignDeclaration, Assignment},
    declaration::ObjectDeclaration,
};

pub mod declare;

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(AssignDeclaration),
    PortMapping(PortMapping),
}

impl From<AssignDeclaration> for Statement {
    fn from(assign: AssignDeclaration) -> Self {
        Statement::Assignment(assign)
    }
}

impl From<PortMapping> for Statement {
    fn from(portmapping: PortMapping) -> Self {
        Statement::PortMapping(portmapping)
    }
}

#[derive(Debug, Clone)]
pub struct PortMapping {
    label: String,
    component_name: String,
    /// The ports, in the order they were declared on the component
    ports: IndexMap<String, ObjectDeclaration>,
    /// Mappings for those ports, will be declared in the order of the original component declaration,
    /// irrespective of the order they're mapped during generation.
    mappings: HashMap<String, AssignDeclaration>,
}

impl PortMapping {
    pub fn from_component(component: &Component, label: impl Into<String>) -> Result<PortMapping> {
        let mut ports = IndexMap::new();
        for port in component.ports() {
            let objs = ObjectDeclaration::from_port(port, false)?;
            for obj in objs {
                ports.insert(obj.identifier().to_string(), obj);
            }
        }
        Ok(PortMapping {
            label: label.into(),
            component_name: component.identifier().to_string(),
            ports,
            mappings: HashMap::new(),
        })
    }

    pub fn ports(&self) -> &IndexMap<String, ObjectDeclaration> {
        &self.ports
    }

    pub fn mappings(&self) -> &HashMap<String, AssignDeclaration> {
        &self.mappings
    }

    pub fn map_port(
        &mut self,
        identifier: impl Into<String>,
        assignment: &(impl Into<Assignment> + Clone),
    ) -> Result<&mut Self> {
        let identifier: &str = &identifier.into();
        let port = self
            .ports()
            .get(identifier)
            .ok_or(Error::InvalidArgument(format!(
                "Port {} does not exist on this component",
                identifier
            )))?;
        let assigned = port.assign(assignment)?;
        self.mappings.insert(identifier.to_string(), assigned);
        Ok(self)
    }

    pub fn finish(self) -> Result<Self> {
        if self.ports().len() == self.mappings().len() {
            Ok(self)
        } else {
            Err(Error::BackEndError(format!(
                "The number of mappings ({}) does not match the number of ports ({})",
                self.mappings().len(),
                self.ports().len()
            )))
        }
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn component_name(&self) -> &str {
        self.component_name.as_str()
    }
}
