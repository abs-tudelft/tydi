//! Standard Library of generated components for Tydi types.
//!
//! The stdlib module is enabled by the `stdlib` feature flag.
//! It contains various useful stream manipulating components
//! and general utilities.
//!
/// TODO: This should be extracted into its own crate.
pub mod basic;
pub mod common;
pub mod utils;

#[cfg(test)]
mod tests {}
