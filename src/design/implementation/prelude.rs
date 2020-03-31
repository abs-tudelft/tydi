//! Prelude module when using Tydi as a Hardware Construction Library in Rust.

pub use crate::design::{Interface, Library, Mode, NamedType, Project, Streamlet, TypeRef};
pub use crate::design::{InterfaceKey, LibraryKey, StreamletKey};
pub use crate::logical::LogicalType;
pub use crate::{Name, Reversed, UniqueKeyBuilder};

pub use crate::design::implementation::structural::{
    Edge, Interfaces, Node, StructuralImpl, StructuralImplBuilder,
};
pub use crate::design::implementation::Implementation;
