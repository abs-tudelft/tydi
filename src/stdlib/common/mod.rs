//! Common properties
//!
//! The goal of this module is to define common traits and functions.

use crate::generator::common::*;

mod entity;
mod architecture;

/// Indicates that a component drives default values
///
/// [Further details: Signal omission](https://abs-tudelft.github.io/tydi/specification/physical.html#signal-omission)
pub trait DrivesDefaults {}

#[cfg(test)]
mod tests {
    // use crate::generator::common::test::{records, test_comp};

    // use super::*;

// pub fn test_entity() -> Entity {
    //     Entity::from(test_comp())
    // }
}