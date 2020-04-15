use std::collections::HashMap;

use crate::design::implementation::Implementation;
use crate::design::{LibKey, Library, Streamlet, StreamletHandle};
use crate::util::UniquelyNamedBuilder;
use crate::{Error, Result};
use crate::{Identify, Name};

/// A collection of Streamlets.
pub struct Project {
    name: Name,
    libraries: HashMap<LibKey, Library>,
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
            libraries: HashMap::new(),
        }
    }

    /// Construct a Project from a UniquelyNamedBuilder with Libraries.
    pub fn from_builder(name: Name, builder: UniquelyNamedBuilder<Library>) -> Result<Self> {
        Ok(Project {
            name,
            libraries: builder
                .finish()?
                .into_iter()
                .map(|lib| (lib.key().clone(), lib))
                .collect::<HashMap<LibKey, Library>>(),
        })
    }

    // Return an iterator over the libraries in this project.
    pub fn libraries(&self) -> impl Iterator<Item = &Library> {
        self.libraries.iter().map(|(_, l)| l)
    }

    pub fn add_lib(&mut self, lib: Library) -> Result<LibKey> {
        let key = lib.key().clone();
        match self.libraries.insert(lib.key().clone(), lib) {
            None => Ok(key),
            Some(_lib) => Err(Error::ProjectError(format!(
                "Error while adding {} to the project",
                key,
            ))),
        }
    }

    pub fn get_lib(&self, lib: LibKey) -> Result<&Library> {
        self.libraries.get(&lib).ok_or_else(|| {
            Error::ProjectError(format!(
                "Error while retrieving {:?}, it does not exist in project.",
                lib
            ))
        })
    }

    pub fn get_lib_mut(&mut self, lib: LibKey) -> Result<&mut Library> {
        self.libraries.get_mut(&lib).ok_or_else(|| {
            Error::ProjectError(format!(
                "Error while retrieving {:?}, it does not exist in project.",
                lib
            ))
        })
    }

    pub fn get_streamlet(&self, streamlet: StreamletHandle) -> Result<&Streamlet> {
        self.get_lib(streamlet.lib())?
            .get_streamlet(streamlet.streamlet())
    }

    pub fn get_streamlet_mut(&mut self, streamlet: StreamletHandle) -> Result<&mut Streamlet> {
        self.get_lib_mut(streamlet.lib())?
            .get_streamlet_mut(streamlet.streamlet())
    }

    /// Add the implementation of a streamlet to the project.
    pub fn add_streamlet_impl(
        &mut self,
        streamlet: StreamletHandle,
        implementation: Implementation,
    ) -> Result<()> {
        self.get_streamlet_mut(streamlet)?
            .attach_implementation(implementation)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Some projects that can be used throughout the crate for testing.
    pub mod proj {
        use super::*;

        pub(crate) fn empty_proj() -> Project {
            Project {
                name: Name::try_new("proj").unwrap(),
                libraries: HashMap::new(),
            }
        }
    }
}
