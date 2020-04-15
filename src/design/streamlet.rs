//! This module contains the Streamlet structure.
//!
//! A streamlet is a component where every [Interface] has a [LogicalType].

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

use crate::design::implementation::composer::GenericComponent;
use crate::design::implementation::Implementation;
use crate::design::{ComponentKey, IFKey};
use crate::logical::LogicalType;
use crate::traits::Identify;
use crate::{Document, Error, Name, Result, Reverse, Reversed, UniqueKeyBuilder};

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
    key: Name,
    /// The mode of the interface.
    mode: Mode,
    /// The type of the interface.
    typ: LogicalType,
    /// Type inference function
    inf_f: Option<Box<fn(LogicalType) -> Result<LogicalType>>>,
    /// The documentation string of the interface, if any.
    doc: Option<String>,
}

impl Identify for Interface {
    fn identifier(&self) -> &str {
        self.key.as_ref()
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
                key: n,
                mode,
                typ: t,
                inf_f: None,
                doc: if let Some(d) = doc {
                    Some(d.to_string())
                } else {
                    None
                },
            }),
        }
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    pub fn with_type_inference(mut self, inf_f: fn(LogicalType) -> Result<LogicalType>) -> Self {
        self.inf_f = Option::from(Box::new(inf_f));
        self
    }

    pub fn infer_type(&mut self, typ: LogicalType) -> Result<()> {
        match &self.inf_f {
            Some(f) => {
                self.typ = f(typ)?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn key(&self) -> &IFKey {
        &self.key
    }

    /// Return the [Mode] of the interface.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Return the [LogicalStreamType] of the interface.
    pub fn typ(&self) -> LogicalType {
        self.typ.clone()
    }
}

impl Reverse for Interface {
    fn reverse(&mut self) {
        self.mode = self.mode.reversed()
    }
}

/// Streamlet interface definition.
#[derive(Clone, Debug)]
pub struct Streamlet {
    /// The name of the streamlet.
    key: Name,
    /// The interfaces of the streamlet.
    interfaces: HashMap<IFKey, Rc<RefCell<Interface>>>,
    /// An optional documentation string for the streamlet to be used by back-ends.
    doc: Option<String>,
    /// Placeholder for future implementation of the streamlet. If this is None, it is a primitive.
    implementation: Option<Rc<Implementation>>,
}

impl PartialEq for Streamlet {
    fn eq(&self, other: &Streamlet) -> bool {
        self.key() == other.key()
    }
}

impl GenericComponent for Streamlet {
    fn key(&self) -> ComponentKey {
        self.key.clone()
    }

    /// Return an iterator over the interfaces of this Streamlet.
    fn interfaces<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        Box::new(self.interfaces.iter().map(|(_, i)| i.borrow()))
    }

    /// Return an iterator over the interfaces of this Streamlet.
    fn interfaces_mut<'a>(&'a self) -> Box<(dyn Iterator<Item = RefMut<Interface>> + 'a)> {
        Box::new(self.interfaces.iter().map(|(_, i)| i.borrow_mut()))
    }

    fn streamlet(&self) -> &Streamlet {
        self
    }

    fn get_interface(&self, key: IFKey) -> Result<Ref<Interface>> {
        match self.interfaces.get(&key) {
            None => Err(Error::InterfaceError(format!(
                "Interface {} does not exist for Streamlet  {}.",
                key,
                self.identifier()
            ))),
            Some(iface) => Ok(iface.borrow()),
        }
    }

    fn get_interface_mut(&self, key: IFKey) -> Result<RefMut<Interface>> {
        match self.interfaces.get(&key) {
            None => Err(Error::InterfaceError(format!(
                "Interface {} does not exist for Streamlet  {}.",
                key,
                self.identifier()
            ))),
            Some(iface) => Ok(iface.borrow_mut()),
        }
    }

    fn get_implementation(&self) -> Option<Rc<Implementation>> {
        self.implementation.clone()
    }
}

impl Streamlet {
    pub fn attach_implementation(&mut self, implementation: Implementation) -> Result<()> {
        self.implementation = Some(Rc::from(implementation));
        Ok(())
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
        builder: UniqueKeyBuilder<Interface>,
        doc: Option<&str>,
    ) -> Result<Self> {
        Ok(Streamlet {
            key: name,
            interfaces: builder
                .finish()?
                .into_iter()
                .map(|iface| (iface.key().clone(), Rc::new(RefCell::new(iface))))
                .collect::<HashMap<IFKey, Rc<RefCell<Interface>>>>(),
            doc: if let Some(d) = doc {
                Some(d.to_string())
            } else {
                None
            },
            implementation: None,
        })
    }

    pub(crate) fn set_key(&mut self, key: ComponentKey) {
        self.key = key;
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
        self.key.as_ref()
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
                UniqueKeyBuilder::new().with_items(vec![
                    Interface::try_new("a", Mode::In, LogicalType::Null, None).unwrap(),
                    Interface::try_new("b", Mode::Out, LogicalType::Null, None).unwrap(),
                ]),
                None,
            )
            .unwrap()
        }
    }
}
