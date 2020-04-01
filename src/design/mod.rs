//! Support for designs that use the Tydi type system.

use crate::logical::LogicalType;
use crate::Name;
use crate::{Error, Result};
use std::convert::TryInto;
use std::fmt::Display;

// Submodules:
pub mod implementation;
pub mod interface;
pub mod library;
pub mod project;
pub mod streamlet;
pub mod typ;

// Re-exports:
pub use interface::{Interface, Mode};
pub use library::Library;
pub use project::Project;
pub use streamlet::Streamlet;
pub use typ::NamedType;

// Structure keys:

/// The key of a type in a library.
pub type TypeKey = Name;

/// The key of an interface on a streamlet.
pub type InterfaceKey = Name;

/// The key of a streamlet in a library.
pub type StreamletKey = Name;

/// The key of a library in a project.
pub type LibraryKey = Name;

// Custom reference types:

/// A reference to a library.
/// A Rust reference can be obtained from a Project.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryRef {
    library: LibraryKey,
}

/// A reference to a named type in some project library.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedTypeRef {
    library: LibraryRef,
    typ: TypeKey,
}

/// A reference to a type.
///
/// If the type is not anonymous, it can be used to look up the corresponding NamedType in a
/// project.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    /// An anonymous type.
    Anon(LogicalType),
    /// A named type.
    Named(NamedTypeRef),
}

impl TypeRef {
    /// Construct a new anonymous type reference.
    pub fn anon(logical_type: LogicalType) -> Self {
        TypeRef::Anon(logical_type)
    }
}

/// A reference to a streamlet.
///
/// An actual Streamlet reference can be obtained from a project.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamletRef {
    library: LibraryRef,
    streamlet: StreamletKey,
}

/// A reference to an interface.
///
/// An actual Interface reference can be obtained from a project.
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceRef {
    streamlet: StreamletRef,
    interface: InterfaceKey,
}

impl NamedTypeRef {
    /// Use with care.
    pub fn try_new<L, T>(lib: L, typ: T) -> Result<Self>
    where
        L: TryInto<LibraryKey>,
        <L as TryInto<LibraryKey>>::Error: Into<Error>,
        T: TryInto<TypeKey>,
        <T as TryInto<TypeKey>>::Error: Into<Error>,
    {
        Ok(NamedTypeRef {
            library: LibraryRef {
                library: lib.try_into().map_err(Into::into)?,
            },
            typ: typ.try_into().map_err(Into::into)?,
        })
    }
}

impl Display for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeRef::Anon(l) => write!(f, "\"{:?}\"", l),
            TypeRef::Named(n) => write!(f, "{}", n),
        }
    }
}

impl Display for NamedTypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.library.library, self.typ)
    }
}

impl Display for StreamletRef {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.library.library, self.streamlet)
    }
}
