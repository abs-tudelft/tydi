use crate::design::StreamletHandle;
use crate::error::Result;
use crate::Name;
use core::fmt::{Debug, Formatter};

///Trait for general implementation backends
pub trait ImplementationBackend {
    fn name(&self) -> Name;
    fn streamlet_handle(&self) -> StreamletHandle;
    fn connect_action(&self) -> Result<()> {
        unimplemented!()
    }
}

impl Debug for dyn ImplementationBackend {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}
