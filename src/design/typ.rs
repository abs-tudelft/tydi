//! Support for named types.

use crate::design::TypeKey;
use crate::logical::LogicalType;
use crate::{Document, Error, Identify, Result, UniqueKeyBuilder};
use indexmap::map::IndexMap;
use std::convert::TryInto;

/// A named Tydi type that has name in a library, usable for type re-use and equality checking.
// TODO: placeholder for actual type implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedType {
    key: TypeKey,
    inner: LogicalType, // placeholder for the actual stuff that needs to be in here.
    doc: Option<String>,
}

impl NamedType {
    pub fn try_new(
        key: impl TryInto<TypeKey, Error = impl Into<Box<dyn std::error::Error>>>,
        typ: LogicalType,
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

    pub fn logical(&self) -> &LogicalType {
        &self.inner
    }
}

impl Identify for NamedType {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

impl Document for NamedType {
    fn doc(&self) -> &Option<String> {
        &self.doc
    }
}

/// Structure to store named types.
// TODO: could be deleted, but I expect we want to manage this separately from the library later on
#[derive(Debug, PartialEq)]
pub struct NamedTypeStore {
    /// A map in which the type can be looked up.
    types: IndexMap<TypeKey, NamedType>,
}

impl Default for NamedTypeStore {
    fn default() -> Self {
        NamedTypeStore {
            types: IndexMap::new(),
        }
    }
}

impl NamedTypeStore {
    pub fn get(&self, key: TypeKey) -> Result<&NamedType> {
        self.types
            .get(&key)
            .ok_or_else(|| Error::ProjectError(format!("Type with key {} does not exist.", key)))
    }

    /// Construct a TypeStore from a UniquelyNamedBuilder.
    ///
    /// The UniquelyNamedBuilder will check whether all Type keys are unique.
    pub fn from_builder(builder: UniqueKeyBuilder<NamedType>) -> Result<Self> {
        Ok(NamedTypeStore {
            types: builder
                .finish()?
                .into_iter()
                .map(|t| (t.key().clone(), t))
                .collect::<IndexMap<TypeKey, NamedType>>(),
        })
    }

    /// Add a type to the TypeStore.
    pub fn insert(&mut self, typ: NamedType) -> Result<TypeKey> {
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

    pub fn types(&self) -> impl Iterator<Item = &NamedType> {
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
