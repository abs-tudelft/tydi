//! Streamlet definition.

use crate::logical::LogicalStreamType;
use crate::{Error, Name};
use std::collections::HashSet;
use std::convert::TryInto;

pub struct StreamletBuilder {
    name: Name,
    interfaces: Vec<Interface>,
}

impl StreamletBuilder {
    pub fn new(name: impl Into<Name>) -> Self {
        StreamletBuilder {
            name: name.into(),
            interfaces: Vec::new(),
        }
    }

    pub fn add_interface(&mut self, interface: Interface) {
        self.interfaces.push(interface);
    }

    pub fn with_interface(mut self, interface: Interface) -> Self {
        self.add_interface(interface);
        self
    }

    pub fn finish(self) -> Result<Streamlet, Error> {
        let set: HashSet<&str> = self
            .interfaces
            .iter()
            .map(|interface| interface.name.as_ref())
            .collect();
        if self.interfaces.len() != set.len() {
            Err(Error::UnexpectedDuplicate)
        } else {
            Ok(Streamlet {
                name: self.name,
                interfaces: self.interfaces,
            })
        }
    }
}

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    name: Name,
    interfaces: Vec<Interface>,
}

impl Streamlet {
    fn get_interface(&self, name: impl AsRef<str>) -> Option<&Interface> {
        self.interfaces
            .iter()
            .find(|interface| interface.name.as_ref() == name.as_ref())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Out,
    In,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    name: Name,
    mode: Mode,
    typ: LogicalStreamType,
}

type BoxedStdError = Box<dyn std::error::Error>;

impl Interface {
    pub fn new(name: Name, mode: Mode, typ: impl Into<LogicalStreamType>) -> Self {
        Interface {
            name,
            mode,
            typ: typ.into(),
        }
    }
    pub fn try_new(
        name: impl TryInto<Name, Error = impl Into<BoxedStdError>>,
        mode: Mode,
        typ: impl TryInto<LogicalStreamType, Error = impl Into<BoxedStdError>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Interface {
            name: name.try_into().map_err(Into::into)?,
            mode,
            typ: typ.try_into().map_err(Into::into)?,
        })
    }
}
