use crate::streamlet::Streamlet;
use crate::util::UniquelyNamedBuilder;
use crate::Name;
use crate::Result;

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
    fn from_builder(name: Name, builder: UniquelyNamedBuilder<Streamlet>) -> Result<Self> {
        Ok(Library {
            name,
            streamlets: builder.finish()?,
        })
    }
}

pub struct Project {
    name: Name,
    libraries: Vec<Library>,
}

impl Project {
    fn from_builder(name: Name, builder: UniquelyNamedBuilder<Library>) -> Result<Self> {
        Ok(Project {
            name,
            libraries: builder.finish()?,
        })
    }
}
