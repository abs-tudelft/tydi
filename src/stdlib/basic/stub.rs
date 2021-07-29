use std::borrow::Borrow;

use log::Log;

use crate::design::implementation::Implementation;
use crate::design::implementation::{composer::GenericComponent, ImplementationBackend};
use crate::design::{Interface, Mode, Project, Streamlet, StreamletHandle, StreamletKey};
use crate::generator::vhdl::VHDLBackEnd;
use crate::generator::GenerateProject;
use crate::logical::{LogicalType, Stream};
use crate::physical::Complexity;
use crate::{Error, Name, Result, UniqueKeyBuilder};
use std::convert::TryFrom;

/// Stub construct, this can be used to prototype a dependency graph
/// or as a basis for custom components.
/// * Passes inputs directly to outputs
/// * Drives defaults where input does not match output
/// * Drives defaults when no input exists, assumes a Null stream at default complexity as input
/// * If output is not specified, uses input stream properties.
#[derive(Clone, Debug)]
pub struct Stub {
    streamlet: Streamlet,
    input_gen: bool,
    output_gen: bool,
}

impl GenericComponent for Stub {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl Stub {
    pub fn try_new(project: &Project, name: Name, op: StreamletHandle) -> Result<Self> {
        let op = project.get_lib(op.lib())?.get_streamlet(op.streamlet())?;
        let mut input_gen: bool = false;
        let mut output_gen: bool = false;

        let stream_in = match op.inputs().next().unwrap().typ() {
            LogicalType::Stream(s) => s,
            _ => {
                log::warn!("No input stream, assuming Null stream.");
                input_gen = true;
                Stream::new_basic(LogicalType::Null)
            }
        };

        let stream_out = match op.outputs().next().unwrap().typ() {
            LogicalType::Stream(s) => s,
            _ => {
                log::warn!("No output stream, using input properties instead.");
                output_gen = true;
                stream_in.clone()
            }
        };

        // TODO: Is there no way to check *just* complexity? Is this already done at an earlier stage?
        // I'd prefer if this didn't fail on incompatible data types.
        // if !LogicalType::Stream(stream_in).compatible(&LogicalType::Stream(stream_out)) {
        //     return Err(Error::ComposerError(format!(
        //         "The input stream is incompatible with the output stream.",
        //     )))
        // }

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(Interface::try_new(
            "in",
            Mode::In,
            stream_in.clone(),
            if input_gen {
                Some("Generated, no input was defined.")
            } else {
                None
            },
        )?);
        ifaces.push(Interface::try_new(
            "out",
            Mode::Out,
            stream_out.clone(),
            if output_gen {
                Some("Generated, no output was defined.")
            } else {
                None
            },
        )?);

        Ok(Stub {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
            input_gen,
            output_gen,
        })
    }

    pub fn with_backend(&mut self, name: Name, streamlet_handle: StreamletHandle) -> Result<()> {
        //self.backend = Option::from(MapStreamBackend { name, streamlet_handle });
        self.streamlet
            .attach_implementation(Implementation::Backend(Box::new(StubBackend {
                name,
                streamlet_handle,
            })))?;
        Ok(())
    }

    pub fn finish(self) -> Stub {
        self
    }
}

pub struct StubBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for StubBackend {
    fn name(&self) -> Name {
        self.name.clone()
    }

    fn streamlet_handle(&self) -> StreamletHandle {
        self.streamlet_handle.clone()
    }

    fn connect_action(&self) -> Result<()> {
        unimplemented!()
    }
}

impl GenerateProject for StubBackend {
    fn generate(&self, project: &Project, path: impl AsRef<std::path::Path>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::fs;

    use crate::Name;
    use crate::Result;
    use crate::design::{LibKey, Library};
    use crate::generator::common::convert::Projectify;
    use crate::stdlib::tests::composition_test_proj;

    use super::*;
    use crate::design::Project;
    use crate::generator::{vhdl::VHDLBackEnd, GenerateProject};

    #[test]
    fn vhdl_impl() -> Result<()> {
        let _folder = fs::create_dir_all("output").unwrap();

        let primitives_lib_key = LibKey::try_from("primitives")?;
        let mut prj = composition_test_proj()?;
        let vhdl = VHDLBackEnd::default();
        let stub = Stub::try_new(
            &prj,
             Name::try_from("stub1")?,
            StreamletHandle{
                lib: primitives_lib_key.clone(),
                streamlet: Name::try_from("test_op")?,
            },
        )?;
        prj.get_lib_mut(primitives_lib_key.clone())?.add_streamlet(stub.streamlet().clone())?;

        assert!(vhdl.generate(&prj, "output").is_ok());

        // println!(
        //     "Map interface {:?}",
        //     vhdl
        // );

        Ok(())
    }
}
