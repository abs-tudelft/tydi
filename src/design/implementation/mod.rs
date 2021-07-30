use std::fmt::{Debug, Formatter};

use crate::design::implementation::composer::{impl_graph::ImplementationGraph, impl_backend::ImplementationBackend};
use crate::design::StreamletHandle;
use crate::Name;
use crate::Result;

pub mod composer;

impl PartialEq for Implementation {
    fn eq(&self, other: &Implementation) -> bool {
        PartialEq::eq(&self.streamlet_handle(), &other.streamlet_handle())
    }
}

/// An implementation variant.
#[derive(Debug)]
pub enum Implementation {
    Structural(ImplementationGraph),
    Backend(Box<dyn ImplementationBackend>),
}

impl Implementation {
    /// Returns a reference to the streamlet this implementation implements.
    pub fn streamlet_handle(&self) -> StreamletHandle {
        match &self {
            Implementation::Structural(s) => s.clone().streamlet(),
            Implementation::Backend(b) => b.streamlet_handle(),
        }
    }
}
