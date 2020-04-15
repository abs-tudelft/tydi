//! Constructs that are used to generate hardware designs, that are not
//! part of the specification (yet).

pub use library::Library;
pub use project::Project;
pub use streamlet::{Interface, Mode, Streamlet};

use crate::Name;

pub mod implementation;
pub mod library;
pub mod param;
pub mod project;
pub mod streamlet;

/// Index types
pub type LibKey = Name;
pub type IFKey = Name;
pub type StreamletKey = Name;
pub type ComponentKey = Name;
pub type ParamKey = Name;
pub type ParamStoreKey = Name;

pub type NodeKey = Name;

pub const THIS_KEY: &str = "this";
pub const GEN_LIB: &str = "gen";

impl NodeKey {
    /// Returns the key that signifies the streamlet that is being implemented itself.
    /// This is a reserved key that users should not be able to use as instance name.
    /// Could be seen as the Rust keyword "self".
    pub fn this() -> NodeKey {
        Name::try_new(THIS_KEY).unwrap()
    }
}

/// Handles for objects inside a project, through project hierarchy
#[derive(Clone, Debug, PartialEq)]
pub struct StreamletHandle {
    pub lib: Name,
    pub streamlet: Name,
}

impl StreamletHandle {
    pub fn lib(&self) -> LibKey {
        self.lib.clone()
    }
    pub fn streamlet(&self) -> StreamletKey {
        self.streamlet.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeIFHandle {
    node: NodeKey,
    iface: IFKey,
}

impl NodeIFHandle {
    pub fn node(&self) -> NodeKey {
        self.node.clone()
    }
    pub fn iface(&self) -> NodeKey {
        self.iface.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParamHandle {
    lib: LibKey,
    param: ParamKey,
}

impl ParamHandle {
    pub fn lib(&self) -> LibKey {
        self.lib.clone()
    }
    pub fn param(&self) -> ParamKey {
        self.param.clone()
    }
}
