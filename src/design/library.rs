//! This module contains the [Library] structure, used to group multiple [Streamlet]s together.
//!
//! This allows users to build up libraries of streamlets and helps to generate language-specific
//! output (e.g. a package in VHDL).

use crate::design::Streamlet;
use crate::error::Error::{FileIOError, ParsingError};
use crate::parser::nom::list_of_streamlets;
use crate::traits::Identify;
use crate::{Name, Result, UniquelyNamedBuilder};
use log::debug;
use std::path::Path;

/// A collection of Streamlets.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    name: Name,
    streamlets: Vec<Streamlet>,
}

impl crate::traits::Identify for Library {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl Library {
    pub fn streamlets(&self) -> Vec<Streamlet> {
        self.streamlets.clone()
    }

    /// Construct a Library from a UniquelyNamedBuilder with Streamlets.
    pub fn from_builder(name: Name, builder: UniquelyNamedBuilder<Streamlet>) -> Result<Self> {
        Ok(Library {
            name,
            streamlets: builder.finish()?,
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
                UniquelyNamedBuilder::new().with_items(streamlets),
            )
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
            Library::from_builder(Name::try_new("test")?, UniquelyNamedBuilder::new()),
        );
        Ok(())
    }

    /// Libraries that can be used for testing purposes throughout the crate.
    pub(crate) mod libs {
        use super::*;

        pub(crate) fn empty_lib() -> Library {
            Library {
                name: Name::try_new("lib").unwrap(),
                streamlets: vec![],
            }
        }
    }
}
