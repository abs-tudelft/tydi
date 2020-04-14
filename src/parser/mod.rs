//! Parser methods and implementations for Tydi designs.
//!
//! The parser module is enabled by the `parser` feature flag. It adds some
//! utitity parser methods and implementations of parsers for Tydi types,
//! streamlets, and libraries.
//!
//! The current parsers are built using [`nom`].
//!
//! [`nom`]: https://crates.io/crates/nom

pub mod ast;
pub mod nom;

#[cfg(test)]
mod tests {}
