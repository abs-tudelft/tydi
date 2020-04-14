//! Structures that represent Tydi interfaces on streamlets.

use crate::design::InterfaceKey;
use crate::logical::LogicalType;
use crate::{Document, Error, Identify, Result, Reverse, Reversed};
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Streamlet interface mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// The interface is an output of the streamlet.
    Out,
    /// The interface is an input of the streamlet.
    In,
}

impl Reverse for Mode {
    fn reverse(&mut self) {
        match self {
            Mode::Out => *self = Mode::In,
            Mode::In => *self = Mode::Out,
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Mode::Out => "out",
                Mode::In => "in",
            }
        )
    }
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
pub struct Interface<'s> {
    /// The key of this interface.
    key: InterfaceKey,
    /// The mode of the interface.
    mode: Mode,
    /// The type of the interface.
    typ: LogicalType<'s>,
    /// The documentation string of the interface, if any.
    doc: Option<String>,
}

impl<'s> Interface<'s> {
    /// Attempt to construct a new interface.
    ///
    /// This function can fail if the key is invalid.
    pub fn try_new(
        key: impl TryInto<InterfaceKey, Error = impl Into<Box<dyn std::error::Error>>>,
        mode: Mode,
        typ: LogicalType<'s>,
        doc: Option<&str>,
    ) -> Result<Self> {
        let n: InterfaceKey = key.try_into().map_err(Into::into)?;
        match n.to_string().as_str() {
            "clk" | "rst" => Err(Error::InterfaceError(format!("Name {} forbidden.", n))),
            _ => Ok(Interface {
                key: n,
                mode,
                typ,
                doc: doc.map(|s| s.to_string()),
            }),
        }
    }

    /// Consume the interface, returning it with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Return the key of this interface
    pub fn key(&self) -> &InterfaceKey {
        &self.key
    }

    /// Return the mode of the interface, i.e. whether it's an input or output.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Return the reference to the type in the project.
    pub fn typ(&self) -> &LogicalType<'s> {
        &self.typ
    }
}

impl<'p> Reverse for Interface<'p> {
    fn reverse(&mut self) {
        self.mode = self.mode.reversed()
    }
}

impl<'p> Identify for Interface<'p> {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

impl<'p> Document for Interface<'p> {
    fn doc(&self) -> &Option<String> {
        &self.doc
    }
}
