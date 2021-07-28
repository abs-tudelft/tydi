use std::borrow::Borrow;

use log::Log;

use crate::design::implementation::Implementation;
use crate::design::{Interface, Mode, Project, Streamlet, StreamletHandle, StreamletKey};
use crate::design::implementation::{ImplementationBackend, composer::GenericComponent};
use crate::physical::Complexity;
use crate::{Error, Name, UniqueKeyBuilder, Result};
use crate::logical::{LogicalType, Stream};
use std::convert::TryFrom;

/// Indicates that a component drives default values
///
/// [Further details: Signal omission](https://abs-tudelft.github.io/tydi/specification/physical.html#signal-omission)
pub trait DrivesDefaults {

}

/// Stub construct, this can be used to prototype a dependency graph
/// or as a basis for custom components.
/// * Passes inputs directly to outputs
/// * Drives defaults where input does not match output
/// * Drives defaults when no input exists, assumes a Null stream at default complexity as input
/// * If output is not specified, uses input stream properties.
#[derive(Clone, Debug)]
pub struct Stub {
    streamlet: Streamlet,
}

impl GenericComponent for Stub {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl Stub {
    pub fn try_new(project: &Project, name: Name, op: StreamletHandle) -> Result<Self> {
        let op = project.get_lib(op.lib())?.get_streamlet(op.streamlet())?;
        let mut input_gen = false;
        let mut output_gen = false;
        let mut drive_defaults = false;

        let stream_in = match op.inputs().next().unwrap().typ() {
            LogicalType::Stream(s) => s,
            _ => {
                log::warn!("No input stream, assuming Null stream.");
                input_gen = true;
                Stream::new_basic(LogicalType::Null)
            },
        };

        let stream_out = match op.outputs().next().unwrap().typ() {
            LogicalType::Stream(s) => s,
            _ => {
                log::warn!("No output stream, using input properties instead.");
                output_gen = true;
                stream_in.clone()
            },
        };

        // TODO: Is there no way to check *just* complexity? Is this already done at an earlier stage?
        // I'd prefer if this didn't fail on incompatible data types.
        if !LogicalType::Stream(stream_in).compatible(&LogicalType::Stream(stream_out)) {
            return Err(Error::ComposerError(format!(
                "The input stream is incompatible with the output stream.",
            )))
        }

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(Interface::try_new(
            "in",
            Mode::In,
            stream_in.clone(),
            if input_gen { Some("Generated, no input was defined.") } else { None },
        )?);
        ifaces.push(Interface::try_new(
            "out",
            Mode::Out,
            stream_out.clone(),
            if output_gen { Some("Generated, no output was defined.") } else { None },
        )?);

        Ok(Stub {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
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
}