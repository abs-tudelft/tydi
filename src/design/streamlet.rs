//! This module contains the Streamlet structure.
//!
//! A streamlet is a component where every [Interface] has a [LogicalType].

use crate::logical::LogicalType;
use crate::traits::Identify;
use crate::util::UniquelyNamedBuilder;
use crate::{Document, Error, Name, Result};
use std::convert::TryInto;
use std::str::FromStr;

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
    /// The name of the interface.
    name: Name,
    /// The mode of the interface.
    mode: Mode,
    /// The type of the interface.
    typ: LogicalType,
    /// The documentation string of the interface, if any.
    doc: Option<String>,
}

impl Interface {
    /// Return the [Mode] of the interface.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Return the [LogicalStreamType] of the interface.
    pub fn typ(&self) -> LogicalType {
        self.typ.clone()
    }
}

impl Identify for Interface {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl Interface {
    /// Try to construct a new interface.
    ///
    /// # Example:
    /// ```
    /// use tydi::logical::LogicalType;
    /// use tydi::design::{Interface, Mode};
    ///
    /// // Define a type.
    /// let a_type = LogicalType::try_new_bits(3);
    /// assert!(a_type.is_ok());
    ///
    /// // Attempt to construct an interface.
    /// let dolphins = Interface::try_new("dolphins",
    ///                                   Mode::In,
    ///                                   a_type.unwrap(),
    ///                                   Some("Look at them swim!"));
    /// assert!(dolphins.is_ok());
    ///
    /// // The names "clk" and "rst" are reserved!
    /// let clk_type = LogicalType::try_new_bits(1);
    /// assert!(clk_type.is_ok());
    /// assert!(Interface::try_new("clk", Mode::In, clk_type.unwrap(), None).is_err());
    /// ```
    pub fn try_new(
        name: impl TryInto<Name, Error = impl Into<Box<dyn std::error::Error>>>,
        mode: Mode,
        typ: impl TryInto<LogicalType, Error = impl Into<Box<dyn std::error::Error>>>,
        doc: Option<&str>,
    ) -> Result<Self> {
        let n: Name = name
            .try_into()
            .map_err(|e| Error::InterfaceError(e.into().to_string()))?;
        let t: LogicalType = typ
            .try_into()
            .map_err(|e| Error::InterfaceError(e.into().to_string()))?;
        match n.to_string().as_str() {
            "clk" | "rst" => Err(Error::InterfaceError(format!("Name {} forbidden.", n))),
            _ => Ok(Interface {
                name: n,
                mode,
                typ: t,
                doc: doc.map(|d| d.to_string()),
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

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    /// The name of the streamlet.
    name: Name,
    /// The interfaces of the streamlet.
    interfaces: Vec<Interface>,
    /// An optional documentation string for the streamlet to be used by back-ends.
    doc: Option<String>,
    /// Placeholder for future implementation of the streamlet. If this is None, it is a primitive.
    implementation: Option<()>,
}

impl Streamlet {
    /// Return an iterator over the interfaces of this Streamlet.
    pub fn interfaces(&self) -> impl Iterator<Item = &Interface> {
        self.interfaces.iter()
    }

    /// Construct a new streamlet from an interface builder that makes sure all interface names
    /// are unique.
    ///
    /// # Example
    /// ```
    /// use tydi::{Name, UniquelyNamedBuilder};
    /// use tydi::logical::LogicalType;
    /// use tydi::design::{Mode, Interface, Streamlet};
    ///
    /// let dough_type = LogicalType::try_new_bits(3);
    /// assert!(dough_type.is_ok());
    /// let dough = Interface::try_new("dough", Mode::In, dough_type.unwrap(), None);
    /// assert!(dough.is_ok());
    /// let cookies_type = LogicalType::try_new_bits(1);
    /// assert!(cookies_type.is_ok());
    /// let cookies = Interface::try_new("cookies", Mode::In, cookies_type.unwrap(), None);
    /// assert!(cookies.is_ok());
    ///    
    /// let my_streamlet = Streamlet::from_builder(
    ///     Name::try_new("baker").unwrap(),
    ///     UniquelyNamedBuilder::new().with_items(vec![dough.unwrap(), cookies.unwrap()]),
    ///     Some("I bake cookies")
    /// );
    /// assert!(my_streamlet.is_ok());
    /// ```
    pub fn from_builder(
        name: Name,
        builder: UniquelyNamedBuilder<Interface>,
        doc: Option<&str>,
    ) -> Result<Self> {
        Ok(Streamlet {
            name,
            interfaces: builder.finish()?,
            doc: doc.map(|d| d.to_string()),
            implementation: None,
        })
    }

    /// Return this streamlet with documentation added.
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
                    Interface::try_new("a", Mode::In, LogicalType::Null, None).unwrap(),
                    Interface::try_new("b", Mode::Out, LogicalType::Null, None).unwrap(),
                ]),
                None,
            )
            .unwrap()
        }
    }
}
