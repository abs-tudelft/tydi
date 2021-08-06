//! Common properties
//!
//! The goal of this module is to define common traits and functions.

use crate::generator::common::*;
use crate::{Document, Identify};

mod impls;

/// Indicates that a component drives default values
///
/// [Further details: Signal omission](https://abs-tudelft.github.io/tydi/specification/physical.html#signal-omission)
pub trait DrivesDefaults {}

/// An Entity.
#[derive(Debug, Clone)]
pub struct Entity {
    /// Component identifier.
    identifier: String,
    /// The parameters of the entity..
    parameters: Vec<Parameter>,
    /// The ports of the entity.
    ports: Vec<Port>,
    /// Documentation.
    doc: Option<String>,
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
    /// Create a new component.
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

    /// Return a reference to the ports of this component.
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    /// Return a reference to the parameters of this component.
    pub fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    /// Return this component with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Set the documentation of this component.
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
    use crate::generator::common::test::{records, test_comp};

    use super::*;

    // pub fn test_entity() -> Entity {
    //     Entity::from(test_comp())
    // }
}