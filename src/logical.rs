//! Logical streams.
//!
//! [Reference]
//!
//! [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html

use crate::{
    physical::{BitCount, Complexity, Fields, PhysicalStream},
    util::log2_ceil,
    Error, Name, NonNegative, PathName, Positive, PositiveReal, Result, Reverse,
};
use indexmap::IndexMap;
use std::{convert::TryInto, iter::FromIterator};

/// A direction of a stream.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html#stream
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Forward indicates that the child stream flows in the same direction as
    /// its parent, complementing the data of its parent in some way.
    Forward,
    /// Reverse indicates that the child stream acts as a response channel for
    /// the parent stream. If there is no parent stream, Forward indicates that
    /// the stream flows in the natural source to sink direction of the logical
    /// stream, while Reverse indicates a control channel in the opposite
    /// direction. The latter may occur for instance when doing random read
    /// access to a memory; the first stream carrying the read commands then
    /// flows in the sink to source direction.
    Reverse,
}

impl Reverse for Direction {
    /// Reverse this direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{Reverse, Reversed, logical::Direction};
    ///
    /// let mut forward = Direction::Forward;
    /// let mut reverse = Direction::Reverse;
    ///
    /// forward.reverse();
    /// assert_eq!(forward, reverse);
    ///
    /// forward.reverse();
    /// assert_eq!(forward, reverse.reversed());
    /// ```
    fn reverse(&mut self) {
        *self = match self {
            Direction::Forward => Direction::Reverse,
            Direction::Reverse => Direction::Forward,
        };
    }
}

/// The synchronicity of the elements in the child stream with respect to the
/// elements in the parent stream.
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
    /// ...
    throughput: PositiveReal,
    /// Nonnegative integer specifying the dimensionality of the child
    /// stream with respect to the parent stream (with no parent, it is the
    /// initial value).
    dimensionality: NonNegative,
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
        throughput: PositiveReal,
        dimensionality: NonNegative,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: Direction,
        user: Option<Box<LogicalStream>>,
        keep: bool,
    ) -> Self {
        Stream {
            data: Box::new(data),
            throughput,
            dimensionality,
            synchronicity,
            complexity: complexity.into(),
            direction,
            user,
            keep,
        }
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn synchronicity(&self) -> Synchronicity {
        self.synchronicity
    }

    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    pub fn throughput(&self) -> PositiveReal {
        self.throughput
    }

    pub fn is_null(&self) -> bool {
        self.data.is_null()
            && (self.user.is_some() && self.user.as_ref().unwrap().is_null())
            && !self.keep
    }

    fn set_throughput(&mut self, throughput: PositiveReal) {
        self.throughput = throughput;
    }

    fn set_synchronicity(&mut self, synchronicity: Synchronicity) {
        self.synchronicity = synchronicity;
    }

    fn set_dimensionality(&mut self, dimensionality: NonNegative) {
        self.dimensionality = dimensionality;
    }
}

impl From<Stream> for LogicalStream {
    fn from(stream: Stream) -> Self {
        LogicalStream::Stream(stream)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group(IndexMap<Name, LogicalStream>);

impl FromIterator<(Name, LogicalStream)> for Group {
    fn from_iter<I: IntoIterator<Item = (Name, LogicalStream)>>(iter: I) -> Self {
        Group(IndexMap::from_iter(iter))
    }
}

impl From<Group> for LogicalStream {
    fn from(group: Group) -> Self {
        LogicalStream::Group(group)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union(IndexMap<Name, LogicalStream>);

impl FromIterator<(Name, LogicalStream)> for Union {
    fn from_iter<I: IntoIterator<Item = (Name, LogicalStream)>>(iter: I) -> Self {
        Union(IndexMap::from_iter(iter))
    }
}

impl From<Union> for LogicalStream {
    fn from(union: Union) -> Self {
        LogicalStream::Union(union)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalStream {
    Null,
    Bits(Positive),
    Group(Group),
    Union(Union),
    Stream(Stream),
}

impl LogicalStream {
    pub fn new_null() -> Self {
        LogicalStream::Null
    }

    pub fn try_new_bits(count: NonNegative) -> Result<Self> {
        Ok(LogicalStream::Bits(Positive::new(count).ok_or_else(
            || Error::InvalidArgument("bit count cannot be zero".to_string()),
        )?))
    }

    pub fn new_bits(count: Positive) -> Self {
        LogicalStream::Bits(count)
    }

    pub fn try_new_group(
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = Error>,
                impl TryInto<LogicalStream, Error = Error>,
            ),
        >,
    ) -> Result<Self> {
        Ok(LogicalStream::Group(
            group
                .into_iter()
                .map(
                    |(name, stream)| match (name.try_into(), stream.try_into()) {
                        (Ok(name), Ok(stream)) => Ok((name, stream)),
                        (Err(name), _) => Err(name),
                        (_, Err(stream)) => Err(stream),
                    },
                )
                .collect::<Result<_>>()?,
        ))
    }

    pub fn new_group(group: impl IntoIterator<Item = (Name, LogicalStream)>) -> Self {
        LogicalStream::Group(group.into_iter().collect())
    }

    // pub fn try_new_union()
    pub fn new_union(union: impl IntoIterator<Item = (Name, LogicalStream)>) -> Self {
        LogicalStream::Union(union.into_iter().collect())
    }

    pub fn new_stream(inner: Stream) -> Self {
        // todo: validation
        LogicalStream::Stream(inner)
    }

    /// Returns true if this logical stream consists of only element-
    /// manipulating nodes. This recursively checks
    pub fn is_element_only(&self) -> bool {
        match self {
            LogicalStream::Null | LogicalStream::Bits(_) => true,
            LogicalStream::Group(Group(fields)) | LogicalStream::Union(Union(fields)) => {
                fields.values().all(|stream| stream.is_element_only())
            }
            LogicalStream::Stream(stream) => stream.data.is_element_only(),
        }
    }

    /// Returns true if and only if this logical stream does not result in any
    /// signals.
    pub fn is_null(&self) -> bool {
        match self {
            LogicalStream::Null => true,
            LogicalStream::Group(Group(fields)) => fields.values().all(|stream| stream.is_null()),
            LogicalStream::Union(Union(fields)) => {
                fields.len() == 1 && fields.values().all(|stream| stream.is_null())
            }
            LogicalStream::Stream(stream) => stream.is_null(),
            LogicalStream::Bits(_) => false,
        }
    }

    pub fn split(&self) -> SplitStream {
        match self {
            LogicalStream::Stream(stream_in) => {
                let mut streams = IndexMap::new();

                let split = stream_in.data.split();
                let (element, rest) = (split.signals, split.streams);
                if !element.is_null()
                    || (stream_in.user.is_some() && stream_in.user.as_ref().unwrap().is_null())
                    || stream_in.keep
                {
                    streams.insert(
                        PathName::new_empty(),
                        // todo: add method
                        Stream::new(
                            element,
                            stream_in.throughput,
                            stream_in.dimensionality,
                            stream_in.synchronicity,
                            stream_in.complexity.clone(),
                            stream_in.direction,
                            stream_in.user.clone(),
                            stream_in.keep,
                        )
                        .into(),
                    );
                }

                streams.extend(rest.into_iter().map(|(name, stream)| match stream {
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
                        stream.set_throughput(stream.throughput * stream_in.throughput);
                        (name, stream.into())
                    }
                    _ => unreachable!(),
                }));

                SplitStream {
                    signals: LogicalStream::Null,
                    streams,
                }
            }
            LogicalStream::Null | LogicalStream::Bits(_) => SplitStream {
                signals: self.clone(),
                streams: IndexMap::new(),
            },
            LogicalStream::Group(Group(fields)) | LogicalStream::Union(Union(fields)) => {
                let signals = fields
                    .into_iter()
                    .map(|(name, stream)| (name.clone(), stream.split().signals))
                    .collect();

                SplitStream {
                    signals: match self {
                        LogicalStream::Group(_) => LogicalStream::Group(Group(signals)),
                        LogicalStream::Union(_) => LogicalStream::Union(Union(signals)),
                        _ => unreachable!(),
                    },
                    streams: fields
                        .into_iter()
                        .map(|(name, stream)| {
                            stream.split().streams.into_iter().map(
                                move |(mut path_name, stream_)| {
                                    path_name.push_back(name.clone());
                                    (path_name, stream_)
                                },
                            )
                        })
                        .flatten()
                        .collect(),
                }
            }
        }
    }

    pub fn fields(&self) -> Fields {
        let mut fields = Fields::new_empty();
        match self {
            LogicalStream::Null | LogicalStream::Stream(_) => fields,
            LogicalStream::Bits(b) => {
                fields.insert(PathName::new_empty(), *b).unwrap();
                fields
            }
            LogicalStream::Group(Group(inner)) => {
                inner.iter().for_each(|(name, stream)| {
                    stream.fields().iter().for_each(|(path_name, bit_count)| {
                        let mut path_name = path_name.clone();
                        path_name.push_back(name.clone());
                        fields.insert(path_name, *bit_count).unwrap();
                    })
                });
                fields
            }
            LogicalStream::Union(Union(inner)) => {
                if inner.len() > 1 {
                    fields
                        .insert(
                            PathName::new(vec!["tag"]).unwrap(),
                            BitCount::new(log2_ceil(
                                BitCount::new(inner.len() as NonNegative).unwrap(),
                            ))
                            .unwrap(),
                        )
                        .unwrap();
                }
                let b = inner.iter().fold(0, |acc, (_, stream)| {
                    acc.max(
                        stream
                            .fields()
                            .values()
                            .fold(0, |acc, count| acc.max(count.get())),
                    )
                });
                if b > 0 {
                    fields
                        .insert(
                            PathName::new(vec!["union"]).unwrap(),
                            BitCount::new(b).unwrap(),
                        )
                        .unwrap();
                }
                fields
            }
        }
    }

    pub fn synthesize(&self) -> SynthesizedStream {
        let split = self.split();
        let (signals, rest) = (split.signals.fields(), split.streams);
        SynthesizedStream {
            signals,
            streams: rest
                .into_iter()
                .map(|(path_name, stream)| match stream {
                    LogicalStream::Stream(stream) => (path_name, stream.into()),
                    _ => unreachable!(),
                })
                .collect(),
        }
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
                LogicalStream::Group(Group(source)) | LogicalStream::Union(Union(source)) => {
                    match other {
                        LogicalStream::Group(Group(sink)) | LogicalStream::Union(Union(sink)) => {
                            source.len() == sink.len()
                                && source.iter().zip(sink.iter()).all(
                                    |((name, stream), (name_, stream_))| {
                                        name == name_ && stream.compatible(&stream_)
                                    },
                                )
                        }
                        _ => false,
                    }
                }
                _ => false,
            }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitStream {
    signals: LogicalStream,
    streams: IndexMap<PathName, LogicalStream>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthesizedStream {
    signals: Fields,
    streams: IndexMap<PathName, PhysicalStream>,
}

impl From<Stream> for PhysicalStream {
    fn from(stream: Stream) -> Self {
        PhysicalStream::new(
            stream.data.fields(),
            Positive::new(stream.throughput.get().ceil() as NonNegative).unwrap(),
            stream.dimensionality,
            stream.complexity,
            stream
                .user
                .map(|stream| stream.fields())
                .unwrap_or_else(Fields::new_empty),
        )
    }
}