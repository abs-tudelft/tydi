//! Parser methods and implementations for Tydi types.
//!
//! The parser module is enabled by the `parser` feature flag. It adds some
//! utitity parser methods and implementations of parsers for Tydi stream and
//! streamlet types, and libraries with streamlets.
//!
//! The current parsers are built using [`nom`].
//!
//! [`nom`]: https://crates.io/crates/nom

pub mod nom;

#[cfg(test)]
mod tests {}
