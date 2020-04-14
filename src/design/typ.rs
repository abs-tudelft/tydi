//! Support for named types.

use crate::design::TypeKey;
use crate::logical::LogicalType;
use crate::{Document, Error, Identify, Result, UniqueKeyBuilder};
use indexmap::map::IndexMap;
use std::convert::TryInto;

/// A named Tydi type that has name in a library, usable for type re-use and equality checking.
// TODO: placeholder for actual type implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedType<'t> {
    key: TypeKey,
    inner: LogicalType<'t>, // placeholder for the actual stuff that needs to be in here.
    doc: Option<String>,
}

impl<'t> NamedType<'t> {
    pub fn try_new(
        key: impl TryInto<TypeKey, Error = impl Into<Box<dyn std::error::Error>>>,
        typ: LogicalType<'t>,
        doc: Option<&str>,
    ) -> Result<Self> {
        let k = key.try_into().map_err(Into::into)?;
        Ok(NamedType {
            key: k,
            inner: typ,
            doc: doc.map(|s| s.to_string()),
        })
    }

    pub fn key(&self) -> &TypeKey {
        &self.key
    }

    pub fn logical(&self) -> &LogicalType<'t> {
        &self.inner
    }
}

impl<'t> Identify for NamedType<'t> {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

impl<'t> Document for NamedType<'t> {
    fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

/// Structure to store named types.
#[derive(Debug, PartialEq)]
pub struct NamedTypeStore<'l> {
    /// A map in which the type can be looked up.
    types: IndexMap<TypeKey, NamedType<'l>>,
}

impl<'l> Default for NamedTypeStore<'l> {
    fn default() -> Self {
        NamedTypeStore {
            types: IndexMap::new(),
        }
    }
}

impl<'l> NamedTypeStore<'l> {
    pub fn get(&self, key: TypeKey) -> Result<&NamedType<'l>> {
        self.types
            .get(&key)
            .ok_or_else(|| Error::ProjectError(format!("Type with key {} does not exist.", key)))
    }

    /// Construct a TypeStore from a UniquelyNamedBuilder.
    ///
    /// The UniquelyNamedBuilder will check whether all Type keys are unique.
    pub fn from_builder(builder: UniqueKeyBuilder<NamedType<'l>>) -> Result<Self> {
        Ok(NamedTypeStore {
            types: builder
                .finish()?
                .into_iter()
                .map(|t| (t.key().clone(), t))
                .collect::<IndexMap<TypeKey, NamedType>>(),
        })
    }

    /// Add a type to the TypeStore.
    pub fn insert(&mut self, typ: NamedType<'l>) -> Result<TypeKey> {
        let key = typ.key().clone();
        if self.types.get(typ.key()).is_some() {
            Err(Error::ProjectError(format!(
                "Type {} already in library.",
                typ.key(),
            )))
        } else {
            self.types.insert(typ.key().clone(), typ);
            Ok(key)
        }
    }

    pub fn types(&self) -> impl Iterator<Item = &NamedType<'l>> {
        self.types.iter().map(|(_, t)| t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_store() {
        let mut ts = NamedTypeStore::default();
        ts.insert(NamedType::try_new("A", LogicalType::Null, None).unwrap())
            .unwrap();
        ts.insert(NamedType::try_new("B", LogicalType::Null, None).unwrap())
            .unwrap();

        // Attempt to insert duplicate:
        assert!(ts
            .insert(NamedType::try_new("A", LogicalType::Null, None).unwrap())
            .is_err());

        assert!(ts.get(TypeKey::try_new("b").unwrap()).is_err());

        // Get a type out of the store:
        assert_eq!(
            ts.get(TypeKey::try_new("B").unwrap()).unwrap(),
            &NamedType::try_new("B", LogicalType::Null, None).unwrap()
        );
    }
}
