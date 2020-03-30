use crate::design::TypeKey;
use crate::logical::LogicalType;
use crate::{Error, Identify, Result, UniqueKeyBuilder};
use indexmap::map::IndexMap;
use std::convert::TryInto;

// TODO: placeholder for actual type implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedType {
    key: TypeKey,
    inner: LogicalType, // placeholder for the actual stuff that needs to be in here.
}

impl NamedType {
    pub fn try_new(
        key: impl TryInto<TypeKey, Error = impl Into<Box<dyn std::error::Error>>>,
        typ: LogicalType,
    ) -> Result<Self> {
        let k = key.try_into().map_err(Into::into)?;
        Ok(NamedType { key: k, inner: typ })
    }
    pub fn key(&self) -> TypeKey {
        self.key.clone()
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
                .map(|t| (t.key(), t))
                .collect::<IndexMap<TypeKey, NamedType>>(),
        })
    }

    /// Add a type to the TypeStore.
    pub fn insert(&mut self, typ: NamedType) -> Result<TypeKey> {
        let key = typ.key();
        if self.types.get(&typ.key()).is_some() {
            Err(Error::ProjectError(format!(
                "Type {} already in library.",
                typ.key(),
            )))
        } else {
            self.types.insert(typ.key(), typ);
            Ok(key)
        }
    }

    pub fn types(&self) -> impl Iterator<Item = &NamedType> {
        self.types.iter().map(|(_, t)| t)
    }
}
