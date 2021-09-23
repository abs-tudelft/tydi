use super::*;

mod impls;

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
