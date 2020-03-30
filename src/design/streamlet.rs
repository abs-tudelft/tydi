//! This module contains the Streamlet structure.

use crate::design::implementation::Implementation;
use crate::design::{Interface, InterfaceKey};
use crate::traits::Identify;
use crate::util::UniqueKeyBuilder;
use crate::{Document, Error, Name, Result};
use indexmap::map::IndexMap;
use std::cell::{Ref, RefCell};
use std::convert::TryInto;
use std::ops::DerefMut;

pub type StreamletKey = Name;

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    /// The name of the streamlet.
    key: StreamletKey,
    /// The interfaces of the streamlet.
    interfaces: IndexMap<InterfaceKey, Interface>,
    /// The implementation of the streamlet.
    implementation: RefCell<Implementation>,
    /// An optional documentation string for the streamlet to be used by back-ends.
    doc: Option<String>,
}

impl Streamlet {
    /// Construct a new streamlet from an interface builder that makes sure all interface names
    /// are unique.
    ///
    /// # Example
    /// ```
    /// ```
    pub fn from_builder(
        key: impl TryInto<StreamletKey, Error = impl Into<Box<dyn std::error::Error>>>,
        builder: UniqueKeyBuilder<Interface>,
        doc: Option<&str>,
    ) -> Result<Self> {
        Ok(Streamlet {
            key: key.try_into().map_err(Into::into)?,
            interfaces: builder
                .finish()?
                .into_iter()
                .map(|i| (i.key(), i))
                .collect::<IndexMap<InterfaceKey, Interface>>(),
            implementation: RefCell::new(Implementation::None),
            doc: if let Some(d) = doc {
                Some(d.to_string())
            } else {
                None
            },
        })
    }

    /// Returns the key of this streamlet.
    pub fn key(&self) -> StreamletKey {
        self.key.clone()
    }

    /// Return this streamlet with documentation added.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Return an iterator over the interfaces of this Streamlet.
    pub fn interfaces(&self) -> impl Iterator<Item = &Interface> {
        self.interfaces.iter().map(|(_, i)| i)
    }

    /// Look up an interface by key, and return it if it exists.
    pub fn get_interface(&self, key: InterfaceKey) -> Result<&Interface> {
        match self.interfaces.get(&key) {
            None => Err(Error::InvalidArgument(format!(
                "Streamlet {} does not have interface {}.",
                self.identifier(),
                key
            ))),
            Some(i) => Ok(i),
        }
    }

    pub fn implementation(&self) -> Ref<Implementation> {
        self.implementation.borrow()
    }

    pub fn set_implementation(&self, implementation: Implementation) -> Result<()> {
        if let Some(r) = implementation.streamlet() {
            if r.streamlet == self.key {
                *self.implementation.borrow_mut().deref_mut() = implementation;
                Ok(())
            } else {
                Err(Error::ProjectError(format!(
                    "Streamlet key {} does not match with provided implementation {}",
                    self.key(),
                    r.streamlet
                )))
            }
        } else {
            Err(Error::ProjectError(format!(
                "Streamlet implementation is not intended for use in {}",
                self.key()
            )))
        }
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
    /// Streamlets that can be used throughout tests.
    pub mod streamlets {}
}
