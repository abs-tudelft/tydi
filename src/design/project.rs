use crate::design::Library;
use crate::util::UniquelyNamedBuilder;
use crate::Name;
use crate::Result;

/// A collection of Streamlets.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    name: Name,
    libraries: Vec<Library>,
}

impl crate::traits::Identify for Project {
    fn identifier(&self) -> &str {
        self.name.as_ref()
    }
}

impl Project {
    /// Construct a Project from a UniquelyNamedBuilder with Libraries.
    pub fn from_builder(name: Name, builder: UniquelyNamedBuilder<Library>) -> Result<Self> {
        Ok(Project {
            name,
            libraries: builder.finish()?,
        })
    }

    // Return a reference to the libraries in this project.
    pub fn libraries(&self) -> Vec<Library> {
        self.libraries.clone()
    }
}
