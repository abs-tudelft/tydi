use std::fmt::Debug;

use crate::design::implementation::composer::impl_backend::ImplementationBackend;
use crate::design::StreamletHandle;

pub mod composer;

impl PartialEq for Implementation {
    fn eq(&self, other: &Implementation) -> bool {
        PartialEq::eq(&self.streamlet_handle(), &other.streamlet_handle())
    }
}

/// An implementation variant.
#[derive(Debug)]
pub enum Implementation {
    Backend(Box<dyn ImplementationBackend>),
}

impl Implementation {
    /// Returns a reference to the streamlet this implementation implements.
    pub fn streamlet_handle(&self) -> StreamletHandle {
        match &self {
            Implementation::Backend(b) => b.streamlet_handle(),
        }
    }
}
