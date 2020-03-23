use crate::design::Library;
use crate::util::UniquelyNamedBuilder;
use crate::Result;
use crate::{Identify, Name};

/// A collection of Streamlets.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    name: Name,
    libraries: Vec<Library>,
}

impl Identify for Project {
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

    // Return an iterator over the libraries in this project.
    pub fn libraries(&self) -> impl Iterator<Item = &Library> {
        self.libraries.iter()
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
                libraries: vec![crate::design::library::tests::libs::empty_lib()],
            }
        }
    }
}
