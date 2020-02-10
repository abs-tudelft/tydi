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

// #[cfg(feature = "data")]
// pub mod data;
pub mod error;
// #[cfg(feature = "generator")]
// pub mod generator;
pub mod logical;
// #[cfg(feature = "parser")]
// pub mod parser;
pub mod physical;
pub mod stream;
// pub mod streamlet;

// #[cfg(feature = "data")]
// pub use data::Data;
pub use logical::LogicalStream;
pub use physical::PhysicalStream;
// pub use streamlet::Streamlet;
