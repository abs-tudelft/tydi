//! Constructs that are used to generate hardware designs, that are not
//! part of the specification (yet).

use crate::Name;
use crate::{Error, Result};

use crate::logical::LogicalType;
pub use interface::{Interface, Mode};
pub use library::Library;
pub use project::Project;
use std::convert::TryInto;
use std::fmt::Display;
pub use streamlet::Streamlet;
pub use typ::NamedType;

pub mod implementation;
pub mod interface;
pub mod library;
pub mod project;
pub mod streamlet;
pub mod structural;
pub mod typ;

// Structure keys:

pub type TypeKey = Name;
pub type InterfaceKey = Name;
pub type StreamletKey = Name;
pub type LibraryKey = Name;

// Custom reference types:

/// A reference to a library.
/// A Rust reference can be obtained from a Project.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryRef {
    library: LibraryKey,
}

/// A reference to a type.
/// A Rust reference can be obtained from a Project if the type is not anonymous.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedTypeRef {
    library: LibraryRef,
    typ: TypeKey,
}

/// A reference to a type.
/// A Rust reference can be obtained from a Project if the type is not anonymous.
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
/// A Rust reference can be obtained from a Project.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamletRef {
    library: LibraryRef,
    streamlet: StreamletKey,
}

/// A reference to an interface.
/// A Rust reference can be obtained from a Project.
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
