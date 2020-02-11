//! Logical streams.
//!
//! [Reference]
//!
//! [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html

use crate::{
    error::Error,
    physical::PhysicalStream,
    stream::{Complexity, Direction, Reverse},
    util::log2_ceil,
};
use indexmap::IndexMap;
use std::{error, num::NonZeroUsize};

/// Specifies the synchronicity of the d-dimensional elements in the child
/// stream with respect to the elements in the parent stream.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Synchronicity {
    /// Indicating that there is a one-to-one relation between the parent and
    /// child elements, and the dimensionality information of the parent stream
    /// is redundantly carried by the child stream as well.
    Sync,
    /// Indicating that there is a one-to-one relation between the parent and
    /// child elements, and the dimensionality information of the parent stream
    /// is omitted in the child stream.
    Flatten,
    /// Desync may be used if the relation between the elements in the child
    /// and parent stream is dependent on context rather than the last flags
    /// in either stream.
    Desync,
    /// FlatDesync, finally, does the same thing as Desync, but also strips the
    /// dimensionality information from the parent. This means there the
    /// relation between the two streams, if any, is fully user-defined.
    FlatDesync,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    /// Any logical stream type representing the data type carried by the
    /// logical stream.
    data: Box<LogicalStream>,
    /// Positive real number used to specify the minimum number of elements
    /// that should be transferrable on the child stream per element in the
    /// parent stream without the child stream becoming the bottleneck
    /// (with no parent, it is the initial value).
    element_lanes: NonZeroUsize,
    /// Nonnegative integer specifying the dimensionality of the child
    /// stream with respect to the parent stream (with no parent, it is the
    /// initial value).
    dimensionality: usize,
    /// The synchronicity of the d-dimensional elements in the child stream
    /// with respect to the elements in the parent stream.
    synchronicity: Synchronicity,
    /// The complexity number for the physical stream interface, as defined
    /// in the physical stream specification.
    complexity: Complexity,
    /// The direction of the stream. If there is no parent stream, this
    /// specifies the direction with respect to the natural direction of
    /// the stream (source to sink).
    direction: Direction,
    /// An optional logical stream type consisting of only
    /// element-manipulating nodes, representing the user data carried by
    /// this logical stream.
    user: Option<Box<LogicalStream>>,
    /// Keep specifies whether the stream carries "extra" information
    /// beyond the data and user signal payloads. x is normally false,
    /// which implies that the Stream node will not result in a physical
    /// stream if both its data and user signals would be empty according
    /// to the rest of this specification; it is effectively optimized
    /// away. Setting keep to true simply overrides this behavior.
    keep: bool,
}

impl Reverse for Stream {
    fn reverse(&mut self) {
        self.direction.reverse();
    }
}

impl Stream {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: LogicalStream,
        lanes: usize,
        dimensionality: usize,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: Direction,
        user: Option<Box<LogicalStream>>,
        keep: bool,
    ) -> Result<Self, Box<dyn error::Error>> {
        Ok(Stream {
            data: Box::new(data),
            element_lanes: NonZeroUsize::new(lanes).ok_or_else(|| {
                Error::InvalidArgument("element lanes cannot be zero".to_string())
            })?,
            dimensionality,
            synchronicity,
            complexity: complexity.into(),
            direction,
            user,
            keep,
        })
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn synchronicity(&self) -> Synchronicity {
        self.synchronicity
    }

    pub fn dimensionality(&self) -> usize {
        self.dimensionality
    }

    pub fn element_lanes(&self) -> usize {
        self.element_lanes.get()
    }

    pub fn is_null(&self) -> bool {
        self.data.is_null()
            && (self.user.is_some() && self.user.as_ref().unwrap().is_null())
            && !self.keep
    }

    fn set_element_lanes(&mut self, element_lanes: NonZeroUsize) {
        self.element_lanes = element_lanes;
    }

    fn set_synchronicity(&mut self, synchronicity: Synchronicity) {
        self.synchronicity = synchronicity;
    }

    fn set_dimensionality(&mut self, dimensionality: usize) {
        self.dimensionality = dimensionality;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalStream {
    Null,
    Bits(NonZeroUsize),
    Group(IndexMap<Option<String>, LogicalStream>),
    Union(IndexMap<Option<String>, LogicalStream>),
    Stream(Stream),
}

impl LogicalStream {
    pub fn new_null() -> Self {
        LogicalStream::Null
    }

    pub fn new_bits(count: usize) -> Result<Self, Box<dyn error::Error>> {
        Ok(LogicalStream::Bits(NonZeroUsize::new(count).ok_or_else(
            || Error::InvalidArgument("bit count cannot be zero".to_string()),
        )?))
    }

    pub fn new_group<T, U>(inner: T) -> Result<Self, Box<dyn error::Error>>
    where
        T: IntoIterator<Item = (Option<String>, LogicalStream)>,
    {
        // todo: validation
        Ok(LogicalStream::Group(inner.into_iter().collect()))
    }

    pub fn new_union(
        inner: impl IntoIterator<Item = (Option<String>, LogicalStream)>,
    ) -> Result<Self, Box<dyn error::Error>> {
        // todo: validation
        Ok(LogicalStream::Union(inner.into_iter().collect()))
    }

    pub fn new_stream(inner: Stream) -> Self {
        // todo: validation
        LogicalStream::Stream(inner)
    }

    /// Returns true if and only if this logical stream does not result in any
    /// signals.
    pub fn is_null(&self) -> bool {
        match self {
            LogicalStream::Stream(stream) => stream.is_null(),
            LogicalStream::Null => true,
            LogicalStream::Group(map) | LogicalStream::Union(map) => {
                map.values().all(|stream| stream.is_null())
            }
            _ => false,
        }
    }

    pub fn split(&self) -> (LogicalStream, IndexMap<Option<String>, LogicalStream>) {
        match self {
            LogicalStream::Stream(stream_in) => {
                let mut map: IndexMap<Option<String>, LogicalStream> = IndexMap::new();

                let (element, rest) = stream_in.data.split();
                if !element.is_null()
                    || (stream_in.user.is_some() && stream_in.user.as_ref().unwrap().is_null())
                    || stream_in.keep
                {
                    map.insert(
                        None,
                        LogicalStream::new_stream(
                            // todo: add method
                            Stream::new(
                                element,
                                stream_in.element_lanes.get(),
                                stream_in.dimensionality,
                                stream_in.synchronicity,
                                stream_in.complexity.clone(),
                                stream_in.direction,
                                stream_in.user.clone(),
                                stream_in.keep,
                            )
                            .unwrap(),
                        ),
                    );
                }

                map.extend(rest.into_iter().map(|(name, stream)| match stream {
                    LogicalStream::Stream(mut stream) => {
                        if stream_in.direction == Direction::Reverse {
                            stream.reverse();
                        }
                        if stream_in.synchronicity == Synchronicity::Flatten
                            || stream_in.synchronicity == Synchronicity::FlatDesync
                        {
                            stream.set_synchronicity(Synchronicity::FlatDesync);
                        }
                        if stream.synchronicity != Synchronicity::Flatten
                            && stream_in.synchronicity != Synchronicity::FlatDesync
                        {
                            stream.set_dimensionality(
                                stream.dimensionality + stream_in.dimensionality,
                            );
                        };
                        stream.set_element_lanes(
                            NonZeroUsize::new(
                                stream.element_lanes.get() * stream_in.element_lanes.get(),
                            )
                            .unwrap(),
                        );
                        (name, LogicalStream::Stream(stream))
                    }
                    _ => unreachable!(),
                }));

                (LogicalStream::Null, map)
            }
            LogicalStream::Null | LogicalStream::Bits(_) => (self.clone(), IndexMap::new()),
            LogicalStream::Group(fields) | LogicalStream::Union(fields) => {
                let signals = fields
                    .into_iter()
                    .map(|(name, stream)| (name.clone(), stream.split().0))
                    .collect();

                (
                    match self {
                        LogicalStream::Group(_) => LogicalStream::Group(signals),
                        LogicalStream::Union(_) => LogicalStream::Union(signals),
                        _ => unreachable!(),
                    },
                    fields
                        .into_iter()
                        .map(|(name, stream)| {
                            stream.split().1.into_iter().map(move |(name_, stream_)| {
                                (
                                    name_
                                        .map(|name_| {
                                            format!("{}__{}", name.as_ref().unwrap(), name_)
                                        })
                                        .or_else(|| name.clone()),
                                    stream_,
                                )
                            })
                        })
                        .flatten()
                        .collect(),
                )
            }
        }
    }

    pub fn fields(&self) -> IndexMap<Option<String>, NonZeroUsize> {
        let mut map = IndexMap::new();
        match self {
            LogicalStream::Null | LogicalStream::Stream(_) => map,
            LogicalStream::Bits(b) => {
                map.insert(None, *b);
                map
            }
            LogicalStream::Group(fields) => {
                map.extend(
                    fields
                        .iter()
                        .map(|(name, stream)| {
                            stream.fields().into_iter().map(move |(name_, count)| {
                                (
                                    name_
                                        .map(|name_| {
                                            format!("{}__{}", name.as_ref().unwrap(), name_)
                                        })
                                        .or_else(|| name.clone()),
                                    count,
                                )
                            })
                        })
                        .flatten(),
                );
                map
            }
            LogicalStream::Union(fields) => {
                if fields.len() > 1 {
                    map.insert(
                        Some("tag".to_string()),
                        NonZeroUsize::new(log2_ceil(NonZeroUsize::new(fields.len()).unwrap()))
                            .unwrap(),
                    );
                }
                let b = fields.iter().fold(0, |acc, (_, stream)| {
                    acc.max(
                        stream
                            .fields()
                            .values()
                            .fold(0, |acc, count| acc.max(count.get())),
                    )
                });
                if b > 0 {
                    map.insert(Some("union".to_string()), NonZeroUsize::new(b).unwrap());
                }
                map
            }
        }
    }

    pub fn synthesize(
        &self,
    ) -> (
        IndexMap<Option<String>, NonZeroUsize>,
        IndexMap<Option<String>, PhysicalStream>,
    ) {
        let (signals, rest) = self.split();
        let signals = signals.fields();
        (
            signals,
            rest.into_iter()
                .map(|(name, stream)| match stream {
                    LogicalStream::Stream(stream) => (name, PhysicalStream::from(stream)),
                    _ => unreachable!(),
                })
                .collect(),
        )
    }

    pub fn compatible(&self, other: &LogicalStream) -> bool {
        self == other
            || match other {
                LogicalStream::Stream(other) => match self {
                    LogicalStream::Stream(stream) => {
                        stream.data.compatible(&other.data) && stream.complexity < other.complexity
                    }
                    _ => false,
                },
                _ => false,
            }
            || match self {
                LogicalStream::Group(source) | LogicalStream::Union(source) => match other {
                    LogicalStream::Group(sink) | LogicalStream::Union(sink) => {
                        source.len() == sink.len()
                            && source.iter().zip(sink.iter()).all(
                                |((name, stream), (name_, stream_))| {
                                    name == name_ && stream.compatible(&stream_)
                                },
                            )
                    }
                    _ => false,
                },
                _ => false,
            }
    }
}

impl From<Stream> for PhysicalStream {
    fn from(stream: Stream) -> Self {
        PhysicalStream::new(
            stream
                .data
                .fields()
                .iter_mut()
                .map(|(name, value)| (name.clone(), value.get()))
                .collect(),
            stream.element_lanes(),
            stream.dimensionality(),
            stream.complexity,
            stream
                .user
                .map(|stream| {
                    stream
                        .fields()
                        .iter_mut()
                        .map(|(name, value)| (name.clone(), value.get()))
                        .collect::<IndexMap<_, _>>()
                })
                .unwrap_or_else(IndexMap::new),
        )
        .unwrap()
    }
}
