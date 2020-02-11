//! # Tydi
//!
//! Tydi is an open specification for complex data structures over hardware
//! streams.
//!
//! # Documentation
//!
//! Documentation is available in the [Tydi book].
//!
//! [Tydi book]: https://abs-tudelft.github.io/tydi/

pub mod error;
// #[cfg(feature = "generator")]
// pub mod generator;
// pub mod logical;
// #[cfg(feature = "parser")]
// pub mod parser;
pub mod physical;
pub mod stream;
// pub mod streamlet;
pub(crate) mod util;

// pub use logical::LogicalStream;
pub use physical::PhysicalStream;
// pub use streamlet::Streamlet;
