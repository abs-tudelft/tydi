use std::borrow::Borrow;
use std::convert::TryFrom;

use crate::design::implementation::composer::GenericComponent;
use crate::design::implementation::Implementation;
use crate::design::{IFKey, Interface, Mode, Project, Streamlet, StreamletHandle, StreamletKey};
use crate::logical::{Direction, LogicalType, Stream, Synchronicity};
use crate::physical::Complexity;
use crate::{Error, Name, NonZeroReal, Result, UniqueKeyBuilder};
use crate::design::implementation::composer::impl_backend::ImplementationBackend;

///! MapStream construct
#[derive(Clone, Debug)]
pub struct MapStream {
    streamlet: Streamlet,
}

impl GenericComponent for MapStream {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl MapStream {
    pub fn try_new(project: &Project, name: Name, op: StreamletHandle) -> Result<Self> {
        let op = project.get_lib(op.lib())?.get_streamlet(op.streamlet())?;

        let op_input_data_type = match op.inputs().next().unwrap().typ() {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for the MapStream pattern required to be be Stream!",
            ))),
        }?;

        let advanced_stream = Stream::new(
            op_input_data_type.data().clone(),
            op_input_data_type.throughput(),
            op_input_data_type.dimensionality() + 1,
            op_input_data_type.synchronicity(),
            Complexity::default(),
            op_input_data_type.direction(),
            //TODO: do we want to pass user signals?
            None,
            //TODO: ?
            false,
        );

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(Interface::try_new(
            "in",
            Mode::In,
            advanced_stream.clone(),
            None,
        )?);
        ifaces.push(Interface::try_new(
            "out",
            Mode::Out,
            advanced_stream.clone(),
            None,
        )?);

        Ok(MapStream {
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
            .attach_implementation(Implementation::Backend(Box::new(MapStreamBackend {
                name,
                streamlet_handle,
            })))?;
        Ok(())
    }

    pub fn finish(self) -> MapStream {
        self
    }
}

pub struct MapStreamBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for MapStreamBackend {
    fn name(&self) -> Name {
        self.name.clone()
    }

    fn streamlet_handle(&self) -> StreamletHandle {
        self.streamlet_handle.clone()
    }
}

///! ReduceStream construct
#[derive(Clone, Debug)]
pub struct ReduceStream {
    streamlet: Streamlet,
}

impl GenericComponent for ReduceStream {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
    fn connect_action(&self) -> Result<()> {
        let input_stream = match self
            .streamlet
            .get_interface(IFKey::try_from("in")?)?
            .typ()
            .clone()
        {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for the ReduceStream pattern required to be be Stream!",
            ))),
        }?;

        let output_stream = Stream::new(
            input_stream.data().clone(),
            input_stream.throughput(),
            input_stream.dimensionality() - 1,
            input_stream.synchronicity(),
            Complexity::default(),
            input_stream.direction(),
            //TODO: do we want to pass user signals?
            None,
            //TODO: ?
            false,
        );

        self.streamlet
            .get_interface_mut(IFKey::try_from("out")?)?
            .infer_type(LogicalType::from(output_stream))
    }
}

impl ReduceStream {
    pub fn try_new(_project: &Project, name: Name, _op: StreamletHandle) -> Result<Self> {
        let input_if = Interface::try_new("in", Mode::In, LogicalType::Null, None)?
            .with_type_inference(|i| {
                match i.clone() {
                    LogicalType::Stream(s) => Ok(s),
                    _ => Err(Error::ComposerError(format!(
                        "The data type for the ReduceStream pattern required to be be Stream!",
                    ))),
                }?;
                Ok(i)
            });

        let output_if = Interface::try_new("out", Mode::Out, LogicalType::Null, None)?
            .with_type_inference(|i| Ok(i));

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(input_if);
        ifaces.push(output_if);

        Ok(ReduceStream {
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
            .attach_implementation(Implementation::Backend(Box::new(ReduceStreamBackend {
                name,
                streamlet_handle,
            })))?;
        Ok(())
    }

    pub fn finish(self) -> ReduceStream {
        self
    }
}

pub struct ReduceStreamBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for ReduceStreamBackend {
    fn name(&self) -> Name {
        self.name.clone()
    }

    fn streamlet_handle(&self) -> StreamletHandle {
        self.streamlet_handle.clone()
    }
}

///! FilterStream construct
#[derive(Clone, Debug)]
pub struct FilterStream {
    streamlet: Streamlet,
}

impl GenericComponent for FilterStream {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
    fn connect_action(&self) -> Result<()> {
        let input_stream = match self
            .streamlet
            .get_interface(IFKey::try_from("in")?)?
            .typ()
            .clone()
        {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for the ReduceStream pattern required to be be Stream!",
            ))),
        }?;

        self.streamlet
            .get_interface_mut(IFKey::try_from("out")?)?
            .infer_type(LogicalType::from(input_stream))
    }
}

impl FilterStream {
    pub fn try_new(_project: &Project, name: Name) -> Result<Self> {
        let input_if = Interface::try_new(
            "in",
            Mode::In,
            Stream::new(
                //Boolean stream
                LogicalType::Null,
                //TODO: What to do with this?
                NonZeroReal::new(1.0 as f64)?,
                0,
                Synchronicity::Sync,
                Complexity::default(),
                Direction::Forward,
                //TODO: do we want to pass user signals?
                None,
                //TODO: ?
                false,
            ),
            None,
        )?
        .with_type_inference(|i| {
            match i.clone() {
                LogicalType::Stream(s) => Ok(s),
                _ => Err(Error::ComposerError(format!(
                    "The data type for the FilterStream pattern required to be be Stream!",
                ))),
            }?;
            Ok(i)
        });

        let output_if = Interface::try_new(
            "out",
            Mode::Out,
            Stream::new(
                //Boolean stream
                LogicalType::Null,
                //TODO: What to do with this?
                NonZeroReal::new(1.0 as f64)?,
                0,
                Synchronicity::Sync,
                Complexity::default(),
                Direction::Forward,
                //TODO: do we want to pass user signals?
                None,
                //TODO: ?
                false,
            ),
            None,
        )?
        .with_type_inference(|i| Ok(i));

        let predicate_stream_type = Stream::new(
            //Boolean stream
            LogicalType::try_new_bits(1)?,
            //TODO: What to do with this?
            NonZeroReal::new(1.0 as f64)?,
            0,
            Synchronicity::Sync,
            Complexity::default(),
            Direction::Forward,
            //TODO: do we want to pass user signals?
            None,
            //TODO: ?
            false,
        );

        let predicate_if = Interface::try_new("pred", Mode::In, predicate_stream_type, None)?;

        let mut ifaces: Vec<Interface> = vec![];
        ifaces.push(input_if);
        ifaces.push(output_if);
        ifaces.push(predicate_if);

        Ok(FilterStream {
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
            .attach_implementation(Implementation::Backend(Box::new(FilterStreamBackend {
                name,
                streamlet_handle,
            })))?;
        Ok(())
    }

    pub fn finish(self) -> FilterStream {
        self
    }
}

pub struct FilterStreamBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for FilterStreamBackend {
    fn name(&self) -> Name {
        self.name.clone()
    }

    fn streamlet_handle(&self) -> StreamletHandle {
        self.streamlet_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::design::StreamletHandle;
    use crate::{Name, Result};
    use super::*;
    use crate::design::implementation::composer::tests::composition_test_proj;

    #[test]
    fn test_map() -> Result<()> {
        let prj = composition_test_proj()?;

        let test_map = MapStream::try_new(
            &prj,
            Name::try_from("test")?,
            StreamletHandle {
                lib: Name::try_from("primitives")?,
                streamlet: Name::try_from("test_op")?,
            },
        )?;
        println!(
            "Map interface {:?}",
            test_map.streamlet().outputs().next().unwrap().typ()
        );

        Ok(())
    }
}
