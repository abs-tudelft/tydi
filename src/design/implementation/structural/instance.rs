//! Streamlet instances for structural implementation.

use crate::design::implementation::structural::NodeKey;
use crate::design::{Interface, InterfaceKey, Project, StreamletRef};
use crate::Result;
use std::fmt::Debug;

/// A structural node representing an instance of a streamlet.
#[derive(Clone, PartialEq)]
pub struct StreamletInst {
    key: NodeKey,
    streamlet: StreamletRef,
    // TODO: Consider if things become much easier if this just contains clones of the relevant
    //  properties of the streamlet.
}

impl StreamletInst {
    /// Construct a new instance.
    pub fn new(key: &NodeKey, streamlet: &StreamletRef) -> Self {
        StreamletInst {
            key: key.clone(),
            streamlet: streamlet.clone(),
        }
    }

    /// Return the key of this instance.
    pub fn key(&self) -> &NodeKey {
        &self.key
    }

    /// Return a reference to the streamlet this instance instantiates.
    pub fn streamlet(&self) -> &StreamletRef {
        &self.streamlet
    }

    pub fn get_interface<'p>(
        &self,
        project: &'p Project<'p>,
        key: InterfaceKey,
    ) -> Result<Interface<'p>> {
        Ok(project
            .get_streamlet(self.streamlet())?
            .get_interface(key)?
            .clone())
    }
}

impl Debug for StreamletInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.key, self.streamlet)
    }
}
