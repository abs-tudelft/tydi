//! This module contains the Library structure, used to group multiple [Streamlet]s together.
//!
//! This allows users to build up libraries of streamlets and helps to generate language-specific
//! output (e.g. a package in VHDL).

use crate::error::Error::{FileIOError, ParsingError};
use crate::parser::nom::list_of_streamlets;
use crate::streamlet::Streamlet;
use crate::util::UniquelyNamedBuilder;
use crate::{Name, Result};
use std::path::Path;

/// A collection of Streamlets.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    name: Name,
    streamlets: Vec<Streamlet>,
}

impl crate::traits::Name for Library {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl Library {
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
            let streamlets: Vec<Streamlet> = list_of_streamlets(
                std::fs::read_to_string(&path)
                    .map_err(|e| FileIOError(e.to_string()))?
                    .as_str(),
            )
            .map_err(|e| ParsingError(e.to_string()))?
            .1;
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
mod test {
    use super::*;

    #[test]
    fn test_library() -> Result<()> {
        let tmpdir = tempfile::tempdir().map_err(|e| FileIOError(e.to_string()))?;
        let path = tmpdir.path().join("test.sdf");
        std::fs::write(path.as_path(), "").map_err(|e| FileIOError(e.to_string()))?;
        assert_eq!(
            Library::from_file(path.as_path()),
            Library::from_builder(Name::try_new("test")?, UniquelyNamedBuilder::new()),
        );
        Ok(())
    }
}
