//! Constructs that are used to generate hardware designs, that are not
//! part of the specification (yet).

pub mod library;
pub mod project;
pub mod streamlet;

pub use library::Library;
pub use project::Project;
pub use streamlet::{Interface, Mode, Streamlet};
