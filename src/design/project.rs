//! Top-level data structure for Tydi designs.

use crate::design::implementation::Implementation;
use crate::design::{Library, LibraryKey, LibraryRef, Streamlet, StreamletRef};
use crate::util::UniqueKeyBuilder;
use crate::{Error, Result};
use crate::{Identify, Name};
use indexmap::map::IndexMap;

/// A container holding multiple libraries that may have references to each other.
///
/// This is the top-level data structure for Tydi generators.
#[derive(Debug, PartialEq)]
pub struct Project<'p> {
    name: Name,
    libraries: IndexMap<LibraryKey, Library<'p>>,
}

impl<'p> Identify for Project<'p> {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'p> Project<'p> {
    /// Construct a new, empty project.
    pub fn new(name: Name) -> Project<'p> {
        Project {
            name,
            libraries: IndexMap::new(),
        }
    }

    /// Construct a project from a set of uniquely named libraries.
    pub fn from_builder(name: Name, builder: UniqueKeyBuilder<Library<'p>>) -> Result<Self> {
        Ok(Project {
            name,
            libraries: builder
                .finish()?
                .into_iter()
                .map(|l| (l.key().clone(), l))
                .collect::<IndexMap<LibraryKey, Library>>(),
        })
    }

    /// Return an iterator over the libraries in this project.
    pub fn libraries(&self) -> impl Iterator<Item = &Library<'p>> {
        self.libraries.iter().map(|(_, l)| l)
    }

    /// Add a library to the project.
    pub fn add_library(&mut self, library: Library<'p>) -> Result<LibraryRef> {
        // Remember the library key.
        let lib_key = library.key().clone();
        // Check if the streamlet already exists.
        if self.libraries.get(library.key()).is_some() {
            Err(Error::ProjectError(format!(
                "Library {} already in project.",
                library.key(),
            )))
        } else {
            // Insert the streamlet and return a reference.
            self.libraries.insert(library.key().clone(), library);
            Ok(LibraryRef { library: lib_key })
        }
    }

    /// Get a library from the project, if it exists. Returns an error otherwise.
    pub fn get_library(&self, library: &LibraryRef) -> Result<&Library<'p>> {
        self.libraries.get(&library.library).ok_or_else(|| {
            Error::ProjectError(format!(
                "Library {:?} does not exist in project {}.",
                library, self.name
            ))
        })
    }

    /// Get a streamlet from the project, if it exists. Returns an error otherwise.
    pub fn get_streamlet(&self, streamlet: &StreamletRef) -> Result<&Streamlet<'p>> {
        self.get_library(streamlet.library())?
            .get_streamlet(streamlet.key())
    }

    /// Add the implementation of a streamlet to the project.
    pub fn add_streamlet_impl(
        &self,
        streamlet: &StreamletRef,
        implementation: Implementation,
    ) -> Result<()> {
        self.get_streamlet(streamlet)?
            .set_implementation(implementation)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Some projects that can be used throughout the crate for testing.
    pub mod proj {
        use super::*;

        /// Return a project with an empty library.
        pub(crate) fn empty_lib_proj<'p>() -> Project<'p> {
            Project::from_builder(
                Name::try_new("test").unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    crate::design::library::tests::libs::empty_lib("empty"),
                ]),
            )
            .unwrap()
        }

        pub(crate) fn single_lib_proj<'p>(name: &str) -> Project<'p> {
            Project::from_builder(
                Name::try_new(name).unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    crate::design::library::tests::libs::simple_lib("my_lib"),
                ]),
            )
            .unwrap()
        }
    }

    #[test]
    fn project_from_builder() {
        assert!(Project::from_builder(
            Name::try_new("test").unwrap(),
            UniqueKeyBuilder::new().with_items(vec![
                crate::design::library::tests::libs::empty_lib("lib"),
                crate::design::library::tests::libs::empty_lib("another"),
            ]),
        )
        .is_ok());
    }

    #[test]
    fn project_errors() {
        let mut prj = proj::empty_lib_proj();
        assert!(prj
            .add_library(crate::design::library::tests::libs::empty_lib("empty"))
            .is_err());
        assert!(prj
            .get_library(&LibraryRef {
                library: Name::try_new("undefined").unwrap()
            })
            .is_err());
    }
}
