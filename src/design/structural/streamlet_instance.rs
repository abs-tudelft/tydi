//! Streamlet instances for structural implementation.

use crate::design::structural::NodeKey;
use crate::design::StreamletRef;
use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub struct StreamletInst {
    key: NodeKey,
    streamlet: StreamletRef,
}

impl StreamletInst {
    pub fn new(key: NodeKey, streamlet: StreamletRef) -> Self {
        StreamletInst { key, streamlet }
    }

    pub fn key(&self) -> NodeKey {
        self.key.clone()
    }

    pub fn streamlet(&self) -> StreamletRef {
        self.streamlet.clone()
    }
}

impl Debug for StreamletInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.key, self.streamlet)
    }
}
