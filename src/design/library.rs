//! This module contains the [Library] structure, used to group multiple [Streamlet]s together.
//!
//! This allows users to build up libraries of streamlets and helps to generate language-specific
//! output (e.g. a package in VHDL).

use std::collections::HashMap;
use std::path::Path;

use log::debug;

use crate::design::implementation::composer::GenericComponent;
use crate::design::param::ParameterStore;
use crate::design::{LibKey, ParamStoreKey, Streamlet, StreamletHandle, StreamletKey};
use crate::error::Error::{FileIOError, ParsingError};
use crate::parser::nom::list_of_streamlets;
use crate::traits::Identify;
use crate::{Error, Name, Result, UniqueKeyBuilder};

/// A collection of Streamlets.
#[derive(PartialEq, Debug)]
pub struct Library {
    key: Name,
    parameter_stores: HashMap<ParamStoreKey, ParameterStore>,
    streamlets: HashMap<StreamletKey, Streamlet>,
}

impl crate::traits::Identify for Library {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

impl Library {
    pub fn streamlets(&self) -> impl Iterator<Item = &Streamlet> {
        self.streamlets.iter().map(|(_, streamlet)| streamlet)
    }

    pub fn new(key: impl Into<LibKey>) -> Library {
        Library {
            key: key.into(),
            parameter_stores: HashMap::new(),
            streamlets: HashMap::new(),
        }
    }

    pub fn try_new(
        key: LibKey,
        parameter_stores: Vec<ParameterStore>,
        streamlets: Vec<Streamlet>,
    ) -> Result<Self> {
        Self::from_builder(
            key,
            UniqueKeyBuilder::new().with_items(parameter_stores),
            UniqueKeyBuilder::new().with_items(streamlets),
        )
    }

    /// Construct a Library from a UniquelyNamedBuilder with Streamlets.
    pub fn from_builder(
        name: LibKey,
        parameter_stores: UniqueKeyBuilder<ParameterStore>,
        streamlets: UniqueKeyBuilder<Streamlet>,
    ) -> Result<Self> {
        Ok(Library {
            key: name,
            parameter_stores: parameter_stores
                .finish()?
                .into_iter()
                .map(|s| (s.key().clone(), s))
                .collect::<HashMap<ParamStoreKey, ParameterStore>>(),
            streamlets: streamlets
                .finish()?
                .into_iter()
                .map(|s| (s.key().clone(), s))
                .collect::<HashMap<StreamletKey, Streamlet>>(),
        })
    }

    /// Construct a Library from a Streamlet Definition File.
    pub fn from_file(path: &Path) -> Result<Self> {
        if path.is_dir() {
            Err(FileIOError(format!(
                "Expected Streamlet Definition File, got directory: \"{}\"",
                path.to_str()
                    .ok_or_else(|| FileIOError("Invalid path.".to_string()))?
            )))
        } else {
            debug!(
                "Parsing: {}",
                path.to_str()
                    .ok_or_else(|| FileIOError("Invalid path.".to_string()))?
            );
            let streamlets: Vec<Streamlet> = list_of_streamlets(
                std::fs::read_to_string(&path)
                    .map_err(|e| FileIOError(e.to_string()))?
                    .as_str(),
            )
            .map_err(|e| ParsingError(e.to_string()))?
            .1;
            debug!("Parsed streamlets: {}", {
                let sln: Vec<&str> = streamlets.iter().map(|s| s.identifier()).collect();
                sln.join(", ")
            });
            Library::from_builder(
                Name::try_new(
                    path.file_stem()
                        .ok_or_else(|| FileIOError("Invalid file name.".to_string()))?
                        .to_str()
                        .unwrap(),
                )?,
                // TODO: No support for parameter groups yet
                UniqueKeyBuilder::new().with_items(vec![]),
                UniqueKeyBuilder::new().with_items(streamlets),
            )
        }
    }

    pub fn key(&self) -> &LibKey {
        &self.key
    }

    pub fn add_streamlet(&mut self, streamlet: Streamlet) -> Result<StreamletHandle> {
        let key = streamlet.key().clone();
        match self.streamlets.insert(streamlet.key().clone(), streamlet) {
            None => Ok(StreamletHandle {
                lib: self.key.clone(),
                streamlet: key.clone(),
            }),
            Some(_lib) => Err(Error::ProjectError(format!(
                "Error while adding {} to the library",
                key,
            ))),
        }
    }
    pub fn get_streamlet(&self, streamlet: StreamletKey) -> Result<&Streamlet> {
        self.streamlets.get(&streamlet).ok_or_else(|| {
            Error::ProjectError(format!(
                "Streamlet {} not found in library {}",
                streamlet,
                self.identifier()
            ))
        })
    }

    pub fn get_streamlet_mut(&mut self, streamlet: StreamletKey) -> Result<&mut Streamlet> {
        match self.streamlets.get_mut(&streamlet) {
            Some(s) => Ok(s),
            None => Err(Error::ProjectError(format!(
                "Streamlet {} not found in library {}",
                streamlet, self.key
            ))),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_library() -> Result<()> {
        let tmpdir = tempfile::tempdir().map_err(|e| FileIOError(e.to_string()))?;
        let path = tmpdir.path().join("test.sdf");
        std::fs::write(path.as_path(), "").map_err(|e| FileIOError(e.to_string()))?;
        assert_eq!(
            Library::from_file(path.as_path()),
            Library::from_builder(
                Name::try_new("test")?,
                UniqueKeyBuilder::new(),
                UniqueKeyBuilder::new()
            ),
        );
        Ok(())
    }

    /// Libraries that can be used for testing purposes throughout the crate.
    pub(crate) mod libs {
        use super::*;

        pub(crate) fn empty_lib() -> Library {
            Library {
                key: Name::try_new("lib").unwrap(),
                parameter_stores: HashMap::new(),
                streamlets: HashMap::new(),
            }
        }
    }
}
