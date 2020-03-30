use crate::design::implementation::Implementation;
use crate::design::{
    Interface, InterfaceRef, Library, LibraryKey, LibraryRef, Streamlet, StreamletRef,
};
use crate::util::UniqueKeyBuilder;
use crate::{Error, Result};
use crate::{Identify, Name};
use indexmap::map::IndexMap;

/// A collection of Streamlets.
#[derive(Debug, PartialEq)]
pub struct Project {
    name: Name,
    libraries: IndexMap<LibraryKey, Library>,
}

impl Identify for Project {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl Project {
    pub fn new(name: Name) -> Project {
        Project {
            name,
            libraries: IndexMap::new(),
        }
    }

    /// Construct a Project from a UniquelyNamedBuilder with Libraries.
    pub fn from_builder(name: Name, builder: UniqueKeyBuilder<Library>) -> Result<Self> {
        Ok(Project {
            name,
            libraries: builder
                .finish()?
                .into_iter()
                .map(|l| (l.key(), l))
                .collect::<IndexMap<LibraryKey, Library>>(),
        })
    }

    // Return an iterator over the libraries in this project.
    pub fn libraries(&self) -> impl Iterator<Item = &Library> {
        self.libraries.iter().map(|(_, l)| l)
    }

    pub fn add_library(&mut self, library: Library) -> Result<LibraryRef> {
        // Remember the library key.
        let lib_key = library.key();
        // Check if the streamlet already exists.
        if self.libraries.get(&library.key()).is_some() {
            Err(Error::ProjectError(format!(
                "Library {} already in project.",
                library.key(),
            )))
        } else {
            // Insert the streamlet and return a reference.
            self.libraries.insert(library.key(), library);
            Ok(LibraryRef { library: lib_key })
        }
    }

    pub fn get_library(&self, library: LibraryRef) -> Result<&Library> {
        self.libraries.get(&library.library).ok_or_else(|| {
            Error::ProjectError(format!(
                "Library {:?} does not exist in project {}.",
                library, self.name
            ))
        })
    }

    pub fn get_streamlet(&self, streamlet: StreamletRef) -> Result<&Streamlet> {
        self.get_library(streamlet.library)?
            .get_streamlet(streamlet.streamlet)
    }

    pub fn get_interface(&self, interface: InterfaceRef) -> Result<&Interface> {
        self.get_streamlet(interface.streamlet)?
            .get_interface(interface.interface)
    }

    pub fn add_streamlet_impl(
        &self,
        streamlet: StreamletRef,
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

        pub(crate) fn empty_proj() -> Project {
            let k = LibraryKey::try_new("lib").unwrap();
            Project {
                name: Name::try_new("proj").unwrap(),
                libraries: vec![(k.clone(), Library::new(k))]
                    .into_iter()
                    .collect::<IndexMap<LibraryKey, Library>>(),
            }
        }
    }
}
