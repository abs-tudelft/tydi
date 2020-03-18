//! This module contains the Streamlet structure.
//!
//! A streamlet is a component where every [Interface] has a [LogicalStreamType].

use crate::logical::LogicalStreamType;
use crate::traits::Identify;
use crate::util::UniquelyNamedBuilder;
use crate::{Document, Error, Name, Result};
use std::convert::TryInto;
use std::str::FromStr;

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    name: Name,
    interfaces: Vec<Interface>,
    doc: Option<String>,
}

impl Streamlet {
    pub fn interfaces(&self) -> Vec<Interface> {
        self.interfaces.clone()
    }

    pub fn from_builder(
        name: Name,
        builder: UniquelyNamedBuilder<Interface>,
        doc: Option<String>,
    ) -> Result<Self> {
        Ok(Streamlet {
            name,
            interfaces: builder.finish()?,
            doc,
        })
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }
}

impl Document for Streamlet {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl Identify for Streamlet {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

/// Streamlet interface mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// The interface is an output of the streamlet.
    Out,
    /// The interface is an input of the streamlet.
    In,
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "in" => Ok(Mode::In),
            "out" => Ok(Mode::Out),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid interface Mode. Expected \"in\" or \"out\"",
                input
            ))),
        }
    }
}

/// A Streamlet interface.
///
/// The names "clk" and "rst" are reserved.
#[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    name: Name,
    mode: Mode,
    typ: LogicalStreamType,
    doc: Option<String>,
}

impl Interface {
    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn typ(&self) -> LogicalStreamType {
        self.typ.clone()
    }
}

impl crate::traits::Identify for Interface {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl Interface {
    pub fn try_new(
        name: impl TryInto<Name, Error = impl Into<Box<dyn std::error::Error>>>,
        mode: Mode,
        typ: impl TryInto<LogicalStreamType, Error = impl Into<Box<dyn std::error::Error>>>,
        doc: Option<String>,
    ) -> Result<Self> {
        let n: Name = name
            .try_into()
            .map_err(|e| Error::InterfaceError(e.into().to_string()))?;
        let t: LogicalStreamType = typ
            .try_into()
            .map_err(|e| Error::InterfaceError(e.into().to_string()))?;
        match n.to_string().as_str() {
            "clk" | "rst" => Err(Error::InterfaceError(format!("Name {} forbidden.", n))),
            _ => Ok(Interface {
                name: n,
                mode,
                typ: t,
                doc,
            }),
        }
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }
}

impl Document for Interface {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Streamlets that can be used throughout tests.
    pub mod streamlets {
        use super::*;

        pub(crate) fn nulls_streamlet(name: impl Into<String>) -> Streamlet {
            Streamlet::from_builder(
                Name::try_new(name).unwrap(),
                UniquelyNamedBuilder::new().with_items(vec![
                    Interface::try_new("a", Mode::In, LogicalStreamType::Null, None).unwrap(),
                    Interface::try_new("b", Mode::Out, LogicalStreamType::Null, None).unwrap(),
                ]),
                None,
            )
            .unwrap()
        }
    }
}
