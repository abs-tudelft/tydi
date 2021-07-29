//! Standard Library of generated components for Tydi types.
//!
//! The stdlib module is enabled by the `stdlib` feature flag.
//! It contains various useful stream manipulating components
//!

pub mod basic;
pub mod common;

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::parser::nom::interface;
    use crate::{Name, Result, UniqueKeyBuilder};
    use crate::design::{LibKey, Library, Project, Streamlet, StreamletKey};

    pub(crate) fn composition_test_proj() -> Result<Project> {
        let key1 = LibKey::try_new("primitives").unwrap();
        let key2 = LibKey::try_new("compositions").unwrap();
        let mut lib = Library::new(key1.clone());

        let mut lib_comp = Library::new(key2.clone());

        let _top = lib_comp
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Top_level").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("in2: in Stream<Bits<1>, d=0>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=0>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _map = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("Magic").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=1>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=1>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let _test_op = lib
            .add_streamlet(
                Streamlet::from_builder(
                    StreamletKey::try_from("test_op").unwrap(),
                    UniqueKeyBuilder::new().with_items(vec![
                        interface("in: in Stream<Bits<32>, d=0>").unwrap().1,
                        interface("out: out Stream<Bits<32>, d=0>").unwrap().1,
                    ]),
                    None,
                )
                    .unwrap(),
            )
            .unwrap();

        let mut prj = Project::new(Name::try_new("TestProj").unwrap());
        prj.add_lib(lib)?;
        prj.add_lib(lib_comp)?;
        Ok(prj)
    }
}