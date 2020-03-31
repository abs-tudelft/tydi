//! This module contains the [Library] structure, used to group multiple [Streamlet]s together.
//!
//! This allows users to build up libraries of streamlets and helps to generate language-specific
//! output (e.g. a package in VHDL).

use crate::design::typ::NamedTypeStore;
use crate::design::{
    LibraryKey, LibraryRef, NamedType, NamedTypeRef, StreamletRef, TypeKey, TypeRef,
};
use crate::{
    design::{Streamlet, StreamletKey},
    parser::nom::list_of_streamlets,
    traits::Identify,
    Error, Name, Result, UniqueKeyBuilder,
};
use indexmap::map::IndexMap;
use log::debug;
use std::path::Path;

/// A library forms a collection of streamlets.
#[derive(Debug, PartialEq)]
pub struct Library {
    key: LibraryKey,
    types: NamedTypeStore,
    streamlets: IndexMap<StreamletKey, Streamlet>,
}

impl Library {
    /// Construct an empty library.
    pub fn new(key: impl Into<LibraryKey>) -> Library {
        Library {
            key: key.into(),
            types: NamedTypeStore::default(),
            streamlets: IndexMap::new(),
        }
    }

    /// Construct a Library from a UniquelyNamedBuilder with Streamlets.
    pub fn from_builder(
        key: LibraryKey,
        types: UniqueKeyBuilder<NamedType>,
        streamlets: UniqueKeyBuilder<Streamlet>,
    ) -> Result<Self> {
        Ok(Library {
            key,
            types: NamedTypeStore::from_builder(types)?,
            streamlets: streamlets
                .finish()?
                .into_iter()
                .map(|s| (s.key(), s))
                .collect::<IndexMap<StreamletKey, Streamlet>>(),
        })
    }

    /// Construct a Library from a Streamlet Definition File (SDF).
    pub fn from_file(path: &Path) -> Result<Self> {
        if path.is_dir() {
            Err(Error::FileIOError(format!(
                "Expected Streamlet Definition File, got directory: \"{}\"",
                path.to_str()
                    .ok_or_else(|| Error::FileIOError("Invalid path.".to_string()))?
            )))
        } else {
            debug!(
                "Parsing: {}",
                path.to_str()
                    .ok_or_else(|| Error::FileIOError("Invalid path.".to_string()))?
            );

            let streamlets: Vec<Streamlet> = list_of_streamlets(
                std::fs::read_to_string(&path)
                    .map_err(|e| Error::FileIOError(e.to_string()))?
                    .as_str(),
            )
            .map_err(|e| Error::ParsingError(e.to_string()))?
            .1;
            debug!("Parsed streamlets: {}", {
                let sln: Vec<&str> = streamlets.iter().map(|s| s.identifier()).collect();
                sln.join(", ")
            });
            let name = Name::try_new(
                path.file_stem()
                    .ok_or_else(|| Error::FileIOError("Invalid file name.".to_string()))?
                    .to_str()
                    .unwrap(),
            )?;
            let streamlets = UniqueKeyBuilder::new().with_items(streamlets);
            // TODO: parse types
            let types = UniqueKeyBuilder::new();

            Library::from_builder(name, types, streamlets)
        }
    }

    pub fn key(&self) -> LibraryKey {
        self.key.clone()
    }

    pub fn this(&self) -> LibraryRef {
        LibraryRef {
            library: self.key(),
        }
    }

    pub fn add_type(&mut self, typ: NamedType) -> Result<TypeRef> {
        // Remember the type key.
        let typ_key = typ.key();
        // Attempt to insert the type.
        self.types.insert(typ)?;
        // Return a TypeRef to the type.
        Ok(TypeRef::Named(NamedTypeRef {
            library: self.this(),
            typ: typ_key,
        }))
    }

    pub fn get_type(&self, key: TypeKey) -> Result<&NamedType> {
        self.types.get(key)
    }

    pub fn add_streamlet(&mut self, streamlet: Streamlet) -> Result<StreamletRef> {
        // Remember the streamlet key.
        let strl_key = streamlet.key();
        // Check if the streamlet already exists.
        if self.streamlets.get(&streamlet.key()).is_some() {
            Err(Error::ProjectError(format!(
                "Streamlet {} already in library.",
                streamlet.key(),
            )))
        } else {
            // Insert the streamlet and return a reference.
            self.streamlets.insert(streamlet.key(), streamlet);
            Ok(StreamletRef {
                library: self.this(),
                streamlet: strl_key,
            })
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

    pub fn streamlets(&self) -> impl Iterator<Item = &Streamlet> {
        self.streamlets.iter().map(|(_, s)| s)
    }

    pub fn named_types(&self) -> impl Iterator<Item = &NamedType> {
        self.types.types()
    }
}

impl Identify for Library {
    fn identifier(&self) -> &str {
        self.key.as_ref()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::logical::LogicalType;
    use crate::Logger;
    use log::LevelFilter;

    /// Libraries that can be used for testing purposes throughout the crate.
    pub(crate) mod libs {
        use super::*;

        pub(crate) fn empty_lib(name: &str) -> Library {
            Library::new(LibraryKey::try_new(name).unwrap())
        }
    }

    #[test]
    fn from_file() -> Result<()> {
        static LOGGER: Logger = Logger;
        log::set_logger(&LOGGER)?;
        log::set_max_level(LevelFilter::Debug);

        let tmpdir = tempfile::tempdir().map_err(|e| Error::FileIOError(e.to_string()))?;
        let path = tmpdir.path().join("test.sdf");
        std::fs::write(path.as_path(), "Streamlet foo (a : In Null, b : Out Null)")
            .map_err(|e| Error::FileIOError(e.to_string()))?;
        assert!(Library::from_file(path.as_path()).is_ok());

        // Attempting to open a directory.
        assert!(dbg!(Library::from_file(tmpdir.path())).is_err());
        // Attempt to open a non-existent file.
        assert!(dbg!(Library::from_file(tmpdir.path().join("asdf").as_path())).is_err());

        Ok(())
    }

    #[test]
    fn library() {
        let mut lib = libs::empty_lib("test");

        lib.add_type(NamedType::try_new("A", LogicalType::Null).unwrap())
            .unwrap();

        lib.add_streamlet(crate::design::streamlet::tests::streamlets::simple("a"))
            .unwrap();

        // attempt to insert duplicate
        assert!(lib
            .add_streamlet(crate::design::streamlet::tests::streamlets::simple("a"))
            .is_err());

        // try some getters
        assert!(lib
            .get_streamlet(StreamletKey::try_new("b").unwrap())
            .is_err());
        assert!(lib.get_type(TypeKey::try_new("B").unwrap()).is_err());

        assert!(lib
            .get_streamlet(StreamletKey::try_new("a").unwrap())
            .is_ok());
        assert!(lib.get_type(TypeKey::try_new("A").unwrap()).is_ok());
    }
}
