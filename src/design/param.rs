use std::collections::HashMap;
use std::convert::TryInto;

///! Generic parameter type
use crate::design::{ParamHandle, ParamKey, ParamStoreKey};
use crate::logical::LogicalType;
use crate::{Document, Error, Identify, Result, UniqueKeyBuilder};

#[derive(Debug, PartialEq)]
pub enum ParameterVariant {
    Type(LogicalType),
    String(String),
    UInt(u32),
    //...
}

#[derive(Debug, PartialEq)]
pub struct NamedParameter {
    key: ParamKey,
    item: ParameterVariant,
    doc: Option<String>,
}

impl NamedParameter {
    pub fn try_new(
        key: impl TryInto<ParamKey, Error = impl Into<Box<dyn std::error::Error>>>,
        item: ParameterVariant,
        doc: Option<&str>,
    ) -> Result<Self> {
        let key = key.try_into().map_err(Into::into)?;
        Ok(NamedParameter {
            key,
            item,
            doc: doc.map(|s| s.to_string()),
        })
    }

    pub fn key(&self) -> &ParamKey {
        &self.key
    }
    pub fn item(&self) -> &ParameterVariant {
        &self.item
    }
}

impl Identify for NamedParameter {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

impl Document for NamedParameter {
    fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterStore {
    key: ParamStoreKey,
    params: HashMap<ParamKey, NamedParameter>,
}

impl Identify for ParameterStore {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

impl ParameterStore {
    pub fn from_builder(
        key: ParamStoreKey,
        builder: UniqueKeyBuilder<NamedParameter>,
    ) -> Result<Self> {
        Ok(ParameterStore {
            key,
            params: builder
                .finish()?
                .into_iter()
                .map(|p| (p.key().clone(), p))
                .collect::<HashMap<ParamKey, NamedParameter>>(),
        })
    }

    pub fn add(&mut self, param: NamedParameter) -> Result<ParamHandle> {
        let key = param.key().clone();
        match self.params.insert(param.key().clone(), param) {
            None => Ok(ParamHandle {
                lib: self.key.clone(),
                param: key.clone(),
            }),
            Some(_lib) => Err(Error::ProjectError(format!(
                "Error while adding {} to the library",
                key,
            ))),
        }
    }

    pub fn get(&self, key: ParamKey) -> Result<&NamedParameter> {
        self.params.get(&key).ok_or_else(|| {
            Error::LibraryError(format!(
                "Parameter {} not found in store {}",
                key,
                self.identifier()
            ))
        })
    }

    pub fn key(&self) -> &ParamStoreKey {
        &self.key
    }
}
