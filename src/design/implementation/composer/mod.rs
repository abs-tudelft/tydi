use std::cell::{Ref, RefMut};
use std::rc::Rc;

use crate::design::implementation::Implementation;
use crate::design::{ComponentKey, IFKey, Interface, Mode, Project, Streamlet};
use crate::generator::dot::DotStyle;
use crate::Result;

pub mod impl_graph;
pub mod misc;
pub mod parser;
pub mod patterns;

/// Traits for components in the implementation graph
pub trait GenHDL {
    fn gen_hdl(&self) -> Result<String>;
}

pub trait GenDot {
    fn gen_dot(
        &self,
        style: &DotStyle,
        project: &Project,
        l: usize,
        prefix: &str,
        label: &str,
    ) -> String;
}

pub trait GenericComponent {
    fn key(&self) -> ComponentKey {
        self.streamlet().key().clone()
    }
    fn interfaces<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        self.streamlet().interfaces()
    }
    fn interfaces_mut<'a>(&'a self) -> Box<(dyn Iterator<Item = RefMut<Interface>> + 'a)> {
        unimplemented!()
    }
    fn streamlet(&self) -> &Streamlet;
    fn inputs<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::In))
    }
    fn outputs<'a>(&'a self) -> Box<(dyn Iterator<Item = Ref<Interface>> + 'a)> {
        Box::new(self.interfaces().filter(|iface| iface.mode() == Mode::Out))
    }
    fn get_interface(&self, key: IFKey) -> Result<Ref<Interface>> {
        self.streamlet().get_interface(key)
    }
    fn get_interface_mut(&self, key: IFKey) -> Result<RefMut<Interface>> {
        self.streamlet().get_interface_mut(key)
    }
    fn get_implementation(&self) -> Option<Rc<Implementation>> {
        self.streamlet().get_implementation()
    }
    fn connect_action(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::fs;
    use crate::design::implementation::composer::parser::ImplParser;
    use crate::design::*;
    use crate::generator::chisel::ChiselBackEnd;
    use crate::generator::dot::DotBackend;
    use crate::generator::GenerateProject;

    use crate::parser::nom::interface;
    use crate::{Name, Result, UniqueKeyBuilder};

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