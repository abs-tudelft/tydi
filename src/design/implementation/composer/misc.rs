/*
pub type FIFODepth = NonNegative;
pub const ElementCountBits: u32 = 16;

pub struct StreamFIFO {
    pub streamlet: Streamlet,
}

impl GenericComponent for StreamFIFO {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl StreamFIFO {
    pub fn try_new(name: &str, data_type: LogicalType, _depth: FIFODepth) -> Result<Self> {
        Ok(StreamFIFO {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    Interface::try_new("in", Mode::In, data_type.clone(), None)?,
                    Interface::try_new("out", Mode::Out, data_type.clone(), None)?,
                ]),
                None,
            )
            .unwrap(),
        })
    }
}

pub struct FlattenStream {
    pub streamlet: Streamlet,
}

impl GenericComponent for FlattenStream {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl FlattenStream {
    pub fn try_new(name: &str, input: Interface) -> Result<Self> {
        let input_data_type = match input.typ().clone() {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for a FlattenStream streamlet required to be be Stream!",
            ))),
        }?;

        if input_data_type.dimensionality() < 1 {
            Err(Error::ComposerError(format!(
                "The dimensionality of the input Stream must be grater than 1! {:?}",
                input_data_type
            )))?
        }

        let output_stream = Stream::new(
            input_data_type.data().clone(),
            input_data_type.throughput(),
            input_data_type.dimensionality()-1,
            input_data_type.synchronicity(),
            Complexity::default(),
            input_data_type.direction().reversed(),
            //TODO: do we want to pass user signals?
            None,
            //TODO: ?
            false,
        );

        Ok(FlattenStream {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    Interface::try_new("in", Mode::In, input_data_type.reversed(), None)?,
                    Interface::try_new("element", Mode::Out, output_stream, None)?,
                    Interface::try_new(
                        "count",
                        Mode::Out,
                        Positive::new(ElementCountBits).unwrap(),
                        None,
                    )?,
                ]),
                None,
            )
            .unwrap(),
        })
    }
}

pub struct SequenceStream {
    pub streamlet: Streamlet,
}

impl GenericComponent for SequenceStream {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl SequenceStream {
    pub fn try_new(name: &str, input: Interface) -> Result<Self> {
        let input_data_type = match input.typ() {
            LogicalType::Stream(s) => Ok(s),
            _ => Err(Error::ComposerError(format!(
                "The data type for a SequenceStream streamlet required to be be Stream!",
            ))),
        }?;

        let output_stream = Stream::new(
            input_data_type.data().clone(),
            input_data_type.throughput(),
            input_data_type.dimensionality()+1,
            input_data_type.synchronicity(),
            Complexity::default(),
            input_data_type.direction().reversed(),
            //TODO: do we want to pass user signals?
            None,
            //TODO: ?
            false,
        );

        Ok(SequenceStream {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(vec![
                    Interface::try_new("element", Mode::In, input_data_type.reversed(), None)?,
                    Interface::try_new("out", Mode::Out, output_stream, None)?,
                    Interface::try_new(
                        "count",
                        Mode::In,
                        Positive::new(ElementCountBits).unwrap(),
                        None,
                    )?,
                ]),
                None,
            )
                .unwrap(),
        })
    }
}

pub struct StreamSync {
    pub streamlet: Streamlet,
}

impl GenericComponent for StreamSync {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl StreamSync {
    pub fn try_new<'a>(name: &str, inputs: impl Iterator<Item = &'a Interface>) -> Result<Self> {
        let mut ifaces: Vec<Interface> = vec![];
        let mut group_members: Vec<(&str, LogicalType)> = vec![];

        for i in inputs {
            ifaces.push(i.clone().reversed());
            group_members.push((i.key().as_ref(), i.typ()));
        }

        let output_data_type = LogicalType::try_new_group(group_members)?;
        let output_stream = Stream::new_basic(output_data_type);
        ifaces.push(Interface::try_new("out", Mode::Out, output_stream, None)?);

        Ok(StreamSync {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
        })
    }
}

pub struct GroupSplit {
    pub streamlet: Streamlet,
}

impl GenericComponent for GroupSplit {
    fn streamlet(&self) -> &Streamlet {
        self.streamlet.borrow()
    }
}

impl GroupSplit {
    pub fn try_new(name: &str, input: Interface, split_interfaces: Vec<PathName>) -> Result<Self> {
        let _haystack: LogicalType = input.typ().clone();
        let mut ifaces: Vec<Interface> =
            vec![Interface::try_new("in", Mode::In, input.typ().clone(), None).unwrap()];

        for item in split_interfaces.iter() {
            let path_string: String = item.0.iter().next().unwrap().to_string();
            let typ: Option<LogicalSplitItem> = input.typ().clone().split().find(|i| {
                i.fields().keys().any(|i| {
                    i.as_ref()
                        .windows(item.len())
                        .any(|name| name == item.as_ref())
                })
            });

            ifaces.push(
                Interface::try_new(
                    path_string,
                    Mode::Out,
                    typ.ok_or_else(|| {
                        Error::ComposerError(format!(
                            "Element {:?} doesn't exist in interface {}",
                            item.as_ref(),
                            input.key().clone()
                        ))
                    })?
                    .logical_type()
                    .clone(),
                    None,
                )
                .unwrap(),
            );
        }
        Ok(GroupSplit {
            streamlet: Streamlet::from_builder(
                StreamletKey::try_from(name).unwrap(),
                UniqueKeyBuilder::new().with_items(ifaces),
                None,
            )
            .unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;




    use crate::design::{
        ComponentKey, IFKey, Interface, Mode, Project, Streamlet, StreamletHandle, StreamletKey,
    };
    use crate::logical::LogicalType;
    use crate::parser::nom::interface;
    use crate::{Result, UniqueKeyBuilder};
    use std::convert::{TryFrom, TryInto};
    /*
        #[test]
        fn dot() {
            let tmpdir = tempfile::tempdir().unwrap();

            let prj = crate::design::project::tests::proj::single_lib_proj("test");
            let dot = DotBackend {};
            // TODO: implement actual test.

            assert!(dot.generate(&prj, tmpdir).is_ok());
        }
    */
    pub(crate) fn nulls_fifo() -> Result<StreamFIFO> {
        StreamFIFO::try_new("Null_fifo", LogicalType::Null, 0)
    }

    #[test]
    fn test_fifo() {
        assert!(nulls_fifo().is_ok())
    }

    #[test]
    fn test_split() -> Result<()> {
        let pn: PathName = "size".try_into().unwrap();
        let test_split = GroupSplit::try_new(
            "test",
            interface("a: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>>")
                .unwrap()
                .1,
            vec![pn],
        );

        for i in test_split.unwrap().outputs() {
            println!("Split interface {}", i.key());
        }
        Ok(())
    }

    #[test]
    fn test_sync() -> Result<()> {
        let streamlet = Streamlet::from_builder(
            StreamletKey::try_from("Top_level").unwrap(),
            UniqueKeyBuilder::new().with_items(vec![
                Interface::try_new("e", Mode::In, LogicalType::Null, None).unwrap(),
                Interface::try_new("f", Mode::Out, LogicalType::Null, None).unwrap(),
                Interface::try_new("g", Mode::Out, LogicalType::Null, None).unwrap(),
                Interface::try_new("h", Mode::Out, LogicalType::Null, None).unwrap(),
            ]),
            None,
        )
        .unwrap();

        let test_sync = StreamSync::try_new("test", streamlet.outputs()).unwrap();

        println!(
            "Sync interface {:?}",
            test_sync.outputs().next().unwrap().typ()
        );

        Ok(())
    }

    #[test]
    fn test_flatten() -> Result<()> {
        let test_iface = interface("a: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>, d=1>")
            .unwrap()
            .1;
        let test_flatten = FlattenStream::try_new("test", test_iface).unwrap();

        println!(
            "Flatten interface {:?}",
            test_flatten.inputs().next().unwrap().typ()
        );

        Ok(())
    }

    #[test]
    fn test_sequence() -> Result<()> {
        let test_iface = interface("a: in Stream<Group<size: Bits<32>, elem: Stream<Bits<32>>>, d=0>")
            .unwrap()
            .1;
        let test_flatten = SequenceStream::try_new("test", test_iface).unwrap();

        println!(
            "Sequence interface {:?}",
            test_flatten.outputs().next().unwrap().typ()
        );

        Ok(())
    }
}*/
