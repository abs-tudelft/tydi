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
/// * If input and output match, passes inputs directly to outputs
/// * If no input exists, acts as a source and drives defaults values to output.
/// * If no output exists, acts as a sink.
#[derive(Clone, Debug)]
pub enum Stub {
    Source(Streamlet),
    Sink(Streamlet),
    Passthrough(Streamlet),
}

impl GenericComponent for Stub {
    fn streamlet(&self) -> &Streamlet {
        match self {
            Stub::Source(s) => s.borrow(),
            Stub::Sink(s) => s.borrow(),
            Stub::Passthrough(s) => s.borrow(),
        }
    }
}

impl Stub {
    pub fn try_new(project: &Project, name: Name, op: StreamletHandle) -> Result<Self> {
        let op = project.get_lib(op.lib())?.get_streamlet(op.streamlet())?;
        let mut is_source: bool = false;
        let mut is_sink: bool = false;

        let stream_in = op.inputs().find_map(|x| match x.typ() {
            LogicalType::Stream(s) => Some(s),
            _ => None
        });

        let stream_out = op.outputs().find_map(|x| match x.typ() {
            LogicalType::Stream(s) => Some(s),
            _ => None
        });

        let mut ifaces: Vec<Interface> = vec![];
        if let Some(s) = stream_in {
            ifaces.push(Interface::try_new("in", Mode::In, s.clone(), None)?);
        } else {
            log::info!("Attempting to implement as source.");
            is_source = true;
        }

        if let Some(s) = stream_out {
            ifaces.push(Interface::try_new("out", Mode::Out, s.clone(), None)?);
        } else if !is_source {
            log::info!("Implementing as sink.");
            is_sink = true;
        } else {
            return Err(Error::ComposerError(format!("No input or output defined.")));
        }

        let streamlet = Streamlet::from_builder(
            StreamletKey::try_from(name).unwrap(),
            UniqueKeyBuilder::new().with_items(ifaces),
            None,
        )?;

        Ok(if is_source {
            Stub::Source(streamlet)
        } else if is_sink {
            Stub::Sink(streamlet)
        } else {
            Stub::Passthrough(streamlet)
        })
    }

    pub fn with_backend(&mut self, name: Name, streamlet_handle: StreamletHandle) -> Result<()> {
        match self {
            Stub::Source(s) => {
                s.attach_implementation(Implementation::Backend(Box::new(SourceStubBackend {
                    name,
                    streamlet_handle,
                })))?
            }
            Stub::Sink(s) => {
                s.attach_implementation(Implementation::Backend(Box::new(SinkStubBackend {
                    name,
                    streamlet_handle,
                })))?
            }
            Stub::Passthrough(s) => s.attach_implementation(Implementation::Backend(Box::new(
                PassthroughStubBackend {
                    name,
                    streamlet_handle,
                },
            )))?,
        }
        Ok(())
    }

    pub fn finish(self) -> Stub {
        self
    }
}

pub struct SourceStubBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

pub struct SinkStubBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

pub struct PassthroughStubBackend {
    name: Name,
    streamlet_handle: StreamletHandle,
}

impl ImplementationBackend for SourceStubBackend {
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

impl ImplementationBackend for SinkStubBackend {
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

impl ImplementationBackend for PassthroughStubBackend {
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

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::design::Library;

    use crate::Result;
    use crate::{parser, Name};

    use super::*;
    use crate::design::Project;

    fn parsed_project() -> Result<Project> {
        let mut prj = Project::new(Name::try_from("test_project")?);
        let (_, source_stub) =
            parser::nom::streamlet("Streamlet source_stub (out_source : out Stream<Bits<1>, d=0>)")
                .unwrap();
        let (_, passthrough_stub) = parser::nom::streamlet(
            "Streamlet passthrough_stub (in_pass : in Stream<Bits<1>, d=0>, out_pass : out Stream<Bits<1>, d=0>)",
        )
        .unwrap();
        let (_, sink_stub) =
            parser::nom::streamlet("Streamlet sink_stub (in_sink : in Stream<Bits<1>, d=0>)")
                .unwrap();
        let lib = Library::try_new(
            Name::try_from("test_library")?,
            vec![],
            vec![source_stub, passthrough_stub, sink_stub],
        )?;
        prj.add_lib(lib)?;
        Ok(prj)
    }

    #[test]
    fn source_stub_interfaces() -> Result<()> {
        let lib_key = Name::try_from("test_library")?;
        let prj = parsed_project()?;
        let source_stub = Stub::try_new(
            &prj,
            Name::try_from("source")?,
            StreamletHandle {
                lib: lib_key.clone(),
                streamlet: Name::try_from("source_stub")?,
            },
        )?;

        // prj.get_lib_mut(lib_key.clone())?.add_streamlet(source_stub.streamlet().clone())?;

        // let _folder = fs::create_dir_all("output")?;
        // let vhdl = VHDLBackEnd::default();
        // vhdl.generate(&prj, "output")?;

        println!("Stub interface\n {:?}\n", source_stub);

        Ok(())
    }

    #[test]
    fn passthrough_stub_interfaces() -> Result<()> {
        let lib_key = Name::try_from("test_library")?;
        let prj = parsed_project()?;
        let stub = Stub::try_new(
            &prj,
            Name::try_from("passthrough")?,
            StreamletHandle {
                lib: lib_key.clone(),
                streamlet: Name::try_from("passthrough_stub")?,
            },
        )?;

        println!("Stub interface\n {:?}\n", stub);

        Ok(())
    }

    #[test]
    fn sink_stub_interfaces() -> Result<()> {
        let lib_key = Name::try_from("test_library")?;
        let prj = parsed_project()?;
        let stub = Stub::try_new(
            &prj,
            Name::try_from("sink")?,
            StreamletHandle {
                lib: lib_key.clone(),
                streamlet: Name::try_from("sink_stub")?,
            },
        )?;

        println!("Stub interface\n {:?}\n", stub);

        Ok(())
    }
}
