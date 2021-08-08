use crate::{Document, Identify};
use crate::generator::common::{Component, Parameter, Port};
use crate::generator::vhdl::Declare;
use crate::stdlib::common::entity::Entity;

impl Declare for Entity {
    fn declare(&self) -> crate::Result<String> {
        let mut result = String::new();
        if let Some(doc) = self.doc() {
            result.push_str("--");
            result.push_str(doc.replace("\n", "\n--").as_str());
            result.push('\n');
        }
        result.push_str(format!("entity {} is\n", self.identifier()).as_str());
        result.push_str(self.ports().declare()?.as_str());
        result.push_str(format!("end {};", self.identifier()).as_str());
        Ok(result)
    }
}

impl Identify for Entity {
    fn identifier(&self) -> &str {
        self.identifier.as_str()
    }
}

impl Document for Entity {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Entity {
    /// Create a new entity.
    pub fn new(
        identifier: impl Into<String>,
        parameters: Vec<Parameter>,
        ports: Vec<Port>,
        doc: Option<String>,
    ) -> Entity {
        Entity {
            identifier: identifier.into(),
            parameters,
            ports,
            doc,
        }
    }

    /// Return a reference to the ports of this entity.
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    /// Return a reference to the parameters of this entity.
    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    /// Return this entity with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this entity.
    pub fn set_doc(&mut self, doc: impl Into<String>) {
        self.doc = Some(doc.into())
    }
}

impl From<Component> for Entity {
    fn from(comp: Component) -> Self {
        Entity::new(comp.identifier(), comp.parameters().to_vec(), comp.ports().to_vec(), comp.doc())
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::common::test::test_comp;
    use crate::generator::vhdl::Declare;
    use crate::stdlib::common::entity::*;

    #[test]
    fn entity_declare() {
        let c = Entity::from(test_comp()).with_doc(" My awesome\n Entity".to_string());
        assert_eq!(
            c.declare().unwrap(),
            concat!(
                "-- My awesome
-- Entity
entity test_comp is
  port(
    a_dn : in a_dn_type;
    a_up : out a_up_type;
    b_dn : out b_dn_type;
    b_up : in b_up_type
  );
end test_comp;"
            )
        );
    }
}
