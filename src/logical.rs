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
use std::str::FromStr;
use std::{
    convert::{TryFrom, TryInto},
    error,
};

/// Direction of a stream.
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

impl Default for Direction {
    fn default() -> Self {
        Direction::Forward
    }
}

impl FromStr for Direction {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "Forward" => Ok(Direction::Forward),
            "Reverse" => Ok(Direction::Reverse),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid Direction",
                input
            ))),
        }
    }
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

impl Default for Synchronicity {
    fn default() -> Self {
        Synchronicity::Sync
    }
}

impl FromStr for Synchronicity {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "Sync" => Ok(Synchronicity::Sync),
            "Flatten" => Ok(Synchronicity::Flatten),
            "Desync" => Ok(Synchronicity::Desync),
            "FlatDesync" => Ok(Synchronicity::FlatDesync),
            _ => Err(Error::InvalidArgument(format!(
                "{} is not a valid Synchronicity",
                input
            ))),
        }
    }
}

/// The stream-manipulating logical stream type.
///
/// Defines a new physical stream.
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#stream)
#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    /// Logical stream type of data elements carried by this stream.
    ///
    /// Any logical stream type representing the data type carried by the
    /// logical stream.
    data: Box<LogicalStreamType>,
    /// Throughput ratio of the stream.
    ///
    /// Positive real number, representing the minimum number of elements that
    /// should be transferrable on the child stream per element in the parent
    /// stream, or if there is no parent stream, the minimum number of elements
    /// that should be transferrable per clock cycle.
    throughput: PositiveReal,
    /// Dimensionality of the stream.
    ///
    /// Nonnegative integer specifying the dimensionality of the child
    /// stream with respect to the parent stream (with no parent, it is the
    /// initial value).
    dimensionality: NonNegative,
    /// Synchronicity of the stream.
    ///
    /// The synchronicity of the d-dimensional elements in the child stream
    /// with respect to the elements in the parent stream.
    synchronicity: Synchronicity,
    /// Complexity level of the stream.
    ///
    /// The complexity number for the physical stream interface, as defined
    /// in the physical stream specification.
    complexity: Complexity,
    /// Direction of the stream.
    ///
    /// The direction of the stream. If there is no parent stream, this
    /// specifies the direction with respect to the natural direction of
    /// the stream (source to sink).
    direction: Direction,
    /// Logical stream type of (optional) user data carried by this stream.
    ///
    /// An optional logical stream type consisting of only
    /// element-manipulating nodes, representing the user data carried by
    /// this logical stream.
    user: Option<Box<LogicalStreamType>>,
    /// Stream carries extra information.
    ///
    /// Keep specifies whether the stream carries "extra" information
    /// beyond the data and user signal payloads. x is normally false,
    /// which implies that the Stream node will not result in a physical
    /// stream if both its data and user signals would be empty according
    /// to the rest of this specification; it is effectively optimized
    /// away. Setting keep to true simply overrides this behavior.
    keep: bool,
}

impl Reverse for Stream {
    /// Reverse the direction of this stream.
    ///
    /// This flips the [`Direction`] of the stream.
    ///
    /// [`Direction`]: ./enum.Direction.html
    fn reverse(&mut self) {
        self.direction.reverse();
    }
}

impl Stream {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: LogicalStreamType,
        throughput: PositiveReal,
        dimensionality: NonNegative,
        synchronicity: Synchronicity,
        complexity: impl Into<Complexity>,
        direction: Direction,
        user: Option<LogicalStreamType>,
        keep: bool,
    ) -> Self {
        Stream {
            data: Box::new(data),
            throughput,
            dimensionality,
            synchronicity,
            complexity: complexity.into(),
            direction,
            user: user.map(Box::new),
            keep,
        }
    }

    /// Returns the direction of this stream.
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the synchronicity of this stream.
    pub fn synchronicity(&self) -> Synchronicity {
        self.synchronicity
    }

    /// Returns the dimensionality of this stream.
    pub fn dimensionality(&self) -> NonNegative {
        self.dimensionality
    }

    /// Returns the throughput ratio of this stream.
    pub fn throughput(&self) -> PositiveReal {
        self.throughput
    }

    /// Returns true if this stream is null i.e. it results in no signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    pub fn is_null(&self) -> bool {
        self.data.is_null()
            && (self.user.is_some() && self.user.as_ref().unwrap().is_null())
            && !self.keep
    }

    /// Set the throughput ratio of this stream.
    fn set_throughput(&mut self, throughput: PositiveReal) {
        self.throughput = throughput;
    }

    /// Set the synchronicity of this stream.
    fn set_synchronicity(&mut self, synchronicity: Synchronicity) {
        self.synchronicity = synchronicity;
    }

    /// Set the dimensionality of this stream.
    fn set_dimensionality(&mut self, dimensionality: NonNegative) {
        self.dimensionality = dimensionality;
    }
}

impl From<Stream> for LogicalStreamType {
    /// Wraps this stream in a [`LogicalStreamType`].
    ///
    /// [`LogicalStreamType`]: ./enum.LogicalStreamType.html
    fn from(stream: Stream) -> Self {
        LogicalStreamType::Stream(stream)
    }
}

/// The Group stream type acts as a product type (composition).
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
#[derive(Debug, Clone, PartialEq)]
pub struct Group(IndexMap<Name, LogicalStreamType>);

impl Group {
    /// Returns a new Group logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalStreamType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, stream) in group
            .into_iter()
            .map(
                |(name, stream)| match (name.try_into(), stream.try_into()) {
                    (Ok(name), Ok(stream)) => Ok((name, stream)),
                    (Err(name), _) => Err(Error::from(name.into())),
                    (_, Err(stream)) => Err(Error::from(stream.into())),
                },
            )
            .collect::<Result<Vec<_>>>()?
        {
            map.insert(name, stream)
                .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
                .transpose()?;
        }
        Ok(Group(map))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Name, &LogicalStreamType)> {
        self.0.iter()
}
}

impl From<Group> for LogicalStreamType {
    /// Wraps this group in a [`LogicalStreamType`].
    ///
    /// [`LogicalStreamType`]: ./enum.LogicalStreamType.html
    fn from(group: Group) -> Self {
        LogicalStreamType::Group(group)
    }
}

///
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
#[derive(Debug, Clone, PartialEq)]
pub struct Union(IndexMap<Name, LogicalStreamType>);

impl Union {
    /// Returns a new Union logical stream type. Returns an error when either
    /// the name or logical stream type conversion fails, or when there are
    /// duplicate names.
    pub fn try_new(
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalStreamType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        let mut map = IndexMap::new();
        for (name, stream) in union
            .into_iter()
            .map(
                |(name, stream)| match (name.try_into(), stream.try_into()) {
                    (Ok(name), Ok(stream)) => Ok((name, stream)),
                    (Err(name), _) => Err(Error::from(name.into())),
                    (_, Err(stream)) => Err(Error::from(stream.into())),
                },
            )
            .collect::<Result<Vec<_>>>()?
        {
            map.insert(name, stream)
                .map(|_| -> Result<()> { Err(Error::UnexpectedDuplicate) })
                .transpose()?;
        }
        Ok(Union(map))
    }
}

impl From<Union> for LogicalStreamType {
    /// Wraps this union in a [`LogicalStreamType`].
    ///
    /// [`LogicalStreamType`]: ./enum.LogicalStreamType.html
    fn from(union: Union) -> Self {
        LogicalStreamType::Union(union)
    }
}

/// Types of logical streams.
///
/// This structure is at the heart of the logical stream specification. It is
/// used both to specify the type of a logical stream and internally for the
/// process of lowering the recursive structure down to physical streams and
/// signals.
///
/// The logical stream type is defined recursively by means of a number of
/// stream types. Two classes of stream types are defined: stream-manipulating
/// types, and element-manipulating types.
///
/// # Examples
///
/// ```rust
/// ```
///
/// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#logical-stream-type)
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalStreamType {
    /// The Null stream type indicates the transferrence of one-valued data: it
    /// is only valid value is ∅ (null).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null)
    Null,
    /// The Bits stream type, defined as `Bits(b)`, indicates the transferrence
    /// of `2^b`-valued data carried by means of a group of `b` bits, where`b`
    /// is a positive integer.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#bits)
    Bits(Positive),
    /// The Group stream type acts as a product type (composition).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#group)
    Group(Group),
    /// The Union stream type acts as a sum type (exclusive disjunction).
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#union)
    Union(Union),
    /// The Stream type is used to define a new physical stream.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#stream)
    Stream(Stream),
}

impl TryFrom<NonNegative> for LogicalStreamType {
    type Error = Error;

    /// Returns a new Bits stream type with the provided bit count as number of
    /// bits. Returns an error when the bit count is zero.
    fn try_from(bit_count: NonNegative) -> Result<Self> {
        LogicalStreamType::try_new_bits(bit_count)
    }
}

impl From<Positive> for LogicalStreamType {
    fn from(bit_count: Positive) -> Self {
        LogicalStreamType::Bits(bit_count)
    }
}

impl LogicalStreamType {
    /// Returns a new Bits stream type with the provided bit count as number of
    /// bits. Returns an error when the bit count is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{Error, logical::LogicalStreamType, Positive};
    ///
    /// let bits = LogicalStreamType::try_new_bits(4);
    /// let zero = LogicalStreamType::try_new_bits(0);
    ///
    /// assert_eq!(bits, Ok(LogicalStreamType::Bits(Positive::new(4).unwrap())));
    /// assert_eq!(zero, Err(Error::InvalidArgument("bit count cannot be zero".to_string())));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_new_bits(bit_count: NonNegative) -> Result<Self> {
        Ok(LogicalStreamType::Bits(
            Positive::new(bit_count)
                .ok_or_else(|| Error::InvalidArgument("bit count cannot be zero".to_string()))?,
        ))
    }

    /// Returns a new Group stream type from the provided iterator of names and
    /// stream types. Returns an error when the values cannot be converted into
    /// valid names, or valid logical stream types as required by [`Group`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{Error, logical::{Group, LogicalStreamType}};
    ///
    /// let group = LogicalStreamType::try_new_group(
    ///     vec![
    ///         ("a", 4), // TryFrom<NonNegative> for LogicalStreamType::Bits.
    ///         ("b", 12),
    ///     ]
    /// )?;
    ///
    /// assert!(match group {
    ///     LogicalStreamType::Group(_) => true,
    ///     _ => false,
    /// });
    ///
    /// assert_eq!(
    ///     LogicalStreamType::try_new_group(vec![("1badname", 4)]),
    ///     Err(Error::InvalidArgument("name cannot start with a digit".to_string()))
    /// );
    /// assert_eq!(
    ///     LogicalStreamType::try_new_group(vec![("good_name", 0)]),
    ///     Err(Error::InvalidArgument("bit count cannot be zero".to_string()))
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`Group`]: ./struct.Group.html
    pub fn try_new_group(
        group: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalStreamType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        Group::try_new(group).map(Into::into)
    }

    pub fn try_new_union(
        union: impl IntoIterator<
            Item = (
                impl TryInto<Name, Error = impl Into<Box<dyn error::Error>>>,
                impl TryInto<LogicalStreamType, Error = impl Into<Box<dyn error::Error>>>,
            ),
        >,
    ) -> Result<Self> {
        Union::try_new(union).map(Into::into)
    }

    /// Returns true if this logical stream consists of only element-
    /// manipulating stream types. This recursively checks all inner stream
    /// types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::logical::LogicalStreamType;
    ///
    /// assert!(LogicalStreamType::Null.is_element_only());
    /// assert!(LogicalStreamType::try_new_bits(3)?.is_element_only());
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_element_only(&self) -> bool {
        match self {
            LogicalStreamType::Null | LogicalStreamType::Bits(_) => true,
            LogicalStreamType::Group(Group(fields)) | LogicalStreamType::Union(Union(fields)) => {
                fields.values().all(|stream| stream.is_element_only())
            }
            LogicalStreamType::Stream(stream) => stream.data.is_element_only(),
        }
    }

    /// Returns true if and only if this logical stream does not result in any
    /// signals.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#null-detection-function)
    pub fn is_null(&self) -> bool {
        match self {
            LogicalStreamType::Null => true,
            LogicalStreamType::Group(Group(fields)) => {
                fields.values().all(|stream| stream.is_null())
            }
            LogicalStreamType::Union(Union(fields)) => {
                fields.len() == 1 && fields.values().all(|stream| stream.is_null())
            }
            LogicalStreamType::Stream(stream) => stream.is_null(),
            LogicalStreamType::Bits(_) => false,
        }
    }

    /// Splits a logical stream type into simplified stream types.
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#split-function)
    pub(crate) fn split(&self) -> SplitStreams {
        match self {
            LogicalStreamType::Stream(stream_in) => {
                let mut streams = IndexMap::new();

                let split = stream_in.data.split();
                let (element, rest) = (split.signals, split.streams);
                if !element.is_null()
                    || (stream_in.user.is_some() && !stream_in.user.as_ref().unwrap().is_null())
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
                            stream_in.user.clone().map(|stream| *stream),
                            stream_in.keep,
                        )
                        .into(),
                    );
                }

                streams.extend(rest.into_iter().map(|(name, stream)| match stream {
                    LogicalStreamType::Stream(mut stream) => {
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

                SplitStreams {
                    signals: LogicalStreamType::Null,
                    streams,
                }
            }
            LogicalStreamType::Null | LogicalStreamType::Bits(_) => SplitStreams {
                signals: self.clone(),
                streams: IndexMap::new(),
            },
            LogicalStreamType::Group(Group(fields)) | LogicalStreamType::Union(Union(fields)) => {
                let signals = fields
                    .into_iter()
                    .map(|(name, stream)| (name.clone(), stream.split().signals))
                    .collect();

                SplitStreams {
                    signals: match self {
                        LogicalStreamType::Group(_) => LogicalStreamType::Group(Group(signals)),
                        LogicalStreamType::Union(_) => LogicalStreamType::Union(Union(signals)),
                        _ => unreachable!(),
                    },
                    streams: fields
                        .into_iter()
                        .map(|(name, stream)| {
                            stream.split().streams.into_iter().map(
                                move |(mut path_name, stream_)| {
                                    path_name.push(name.clone());
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

    /// Flattens a logical stream type consisting of Null, Bits, Group and
    /// Union stream types into a [`Fields`].
    ///
    /// [Reference](https://abs-tudelft.github.io/tydi/specification/logical.html#field-conversion-function)
    ///
    /// [`Fields`]: ./struct.Fields.html
    pub fn fields(&self) -> Fields {
        let mut fields = Fields::new_empty();
        match self {
            LogicalStreamType::Null | LogicalStreamType::Stream(_) => fields,
            LogicalStreamType::Bits(b) => {
                fields.insert(PathName::new_empty(), *b).unwrap();
                fields
            }
            LogicalStreamType::Group(Group(inner)) => {
                inner.iter().for_each(|(name, stream)| {
                    stream.fields().iter().for_each(|(path_name, bit_count)| {
                        fields
                            .insert(path_name.with_parent(name.clone()), *bit_count)
                            .unwrap();
                    })
                });
                fields
            }
            LogicalStreamType::Union(Union(inner)) => {
                if inner.len() > 1 {
                    fields
                        .insert(
                            PathName::try_new(vec!["tag"]).unwrap(),
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
                            PathName::try_new(vec!["union"]).unwrap(),
                            BitCount::new(b).unwrap(),
                        )
                        .unwrap();
                }
                fields
            }
        }
    }

    pub fn synthesize(&self) -> LogicalStream {
        let split = self.split();
        let (signals, rest) = (split.signals.fields(), split.streams);
        LogicalStream {
            signals,
            streams: rest
                .into_iter()
                .map(|(path_name, stream)| match stream {
                    LogicalStreamType::Stream(stream) => (
                        path_name,
                        PhysicalStream::new(
                            stream.data.fields(),
                            Positive::new(stream.throughput.get().ceil() as NonNegative).unwrap(),
                            stream.dimensionality,
                            stream.complexity,
                            stream
                                .user
                                .map(|stream| stream.fields())
                                .unwrap_or_else(Fields::new_empty),
                        ),
                    ),
                    _ => unreachable!(),
                })
                .collect(),
        }
    }

    pub fn compatible(&self, other: &LogicalStreamType) -> bool {
        self == other
            || match other {
                LogicalStreamType::Stream(other) => match self {
                    LogicalStreamType::Stream(stream) => {
                        stream.data.compatible(&other.data) && stream.complexity < other.complexity
                    }
                    _ => false,
                },
                _ => false,
            }
            || match self {
                LogicalStreamType::Group(Group(source))
                | LogicalStreamType::Union(Union(source)) => match other {
                    LogicalStreamType::Group(Group(sink))
                    | LogicalStreamType::Union(Union(sink)) => {
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SplitStreams {
    signals: LogicalStreamType,
    streams: IndexMap<PathName, LogicalStreamType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalStream {
    signals: Fields,
    streams: IndexMap<PathName, PhysicalStream>,
}

impl LogicalStream {
    pub fn streams(&self) -> impl Iterator<Item = (&PathName, &PhysicalStream)> {
        self.streams.iter()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Module containing functions that return common LogicalStreamTypes to be used for testing
    /// purposed only.
    pub(crate) mod streams {
        use super::*;

        use crate::logical::LogicalStreamType;

        #[allow(dead_code)]
        pub(crate) fn null() -> LogicalStreamType {
            LogicalStreamType::from(Stream::new(
                LogicalStreamType::Null,
                PositiveReal::new(1.).unwrap(),
                0,
                Synchronicity::Sync,
                Complexity::default(),
                Direction::Forward,
                None,
                false,
            ))
        }

        pub(crate) fn single_element() -> LogicalStreamType {
            LogicalStreamType::from(Stream::new(
                LogicalStreamType::try_new_bits(8).unwrap(),
                PositiveReal::new(1.).unwrap(),
                0,
                Synchronicity::Sync,
                Complexity::default(),
                Direction::Forward,
                None,
                false,
            ))
        }

        #[allow(dead_code)]
        pub(crate) fn nested_elements() -> LogicalStreamType {
            LogicalStreamType::from(Stream::new(
                LogicalStreamType::try_new_group(vec![
                    ("a", LogicalStreamType::try_new_bits(1).unwrap()),
                    (
                        "b",
                        LogicalStreamType::try_new_group(vec![
                            ("c", LogicalStreamType::try_new_bits(2).unwrap()),
                            ("d", LogicalStreamType::try_new_bits(3).unwrap()),
                        ])
                        .unwrap(),
                    ),
                ])
                .unwrap(),
                PositiveReal::new(1.).unwrap(),
                0,
                Synchronicity::Sync,
                Complexity::default(),
                Direction::Forward,
                None,
                false,
            ))
        }
    }

    #[test]
    fn union() -> Result<()> {
        let b = LogicalStreamType::try_new_group(vec![("x", 2), ("y", 2)])?;
        let c = Stream::new(
            LogicalStreamType::Bits(Positive::new(4).unwrap()),
            PositiveReal::new(1.).unwrap(),
            1,
            Synchronicity::Sync,
            1,
            Direction::Forward,
            None,
            false,
        );
        let u = LogicalStreamType::try_new_union(vec![
            ("a", 3.try_into()?),
            ("b", b.clone()),
            ("c", LogicalStreamType::Stream(c)),
        ])?;
        let stream: LogicalStreamType = Stream::new(
            u,
            PositiveReal::new(1.).unwrap(),
            1,
            Synchronicity::Sync,
            1,
            Direction::Forward,
            None,
            false,
        )
        .into();

        let logical_stream = stream.synthesize();
        assert_eq!(logical_stream.streams.len(), 2);
        assert_eq!(
            logical_stream.streams.keys().collect::<Vec<_>>(),
            vec![&PathName::new_empty(), &PathName::try_new(vec!["c"])?]
        );
        assert_eq!(
            logical_stream
                .streams
                .values()
                .map(|physical_stream| physical_stream.element_fields().iter())
                .flatten()
                .collect::<Vec<_>>(),
            vec![
                (&PathName::try_new(vec!["tag"])?, &Positive::new(2).unwrap()),
                (
                    &PathName::try_new(vec!["union"])?,
                    &Positive::new(3).unwrap()
                ),
                (&PathName::new_empty(), &Positive::new(4).unwrap()),
            ]
        );
        assert_eq!(
            logical_stream
                .streams
                .values()
                .map(|physical_stream| physical_stream.dimensionality())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        let c = Stream::new(
            LogicalStreamType::Bits(Positive::new(4).unwrap()),
            PositiveReal::new(1.).unwrap(),
            1,
            Synchronicity::Flatten,
            1,
            Direction::Forward,
            None,
            false,
        );
        let u = LogicalStreamType::try_new_union(vec![
            ("a", 3.try_into()?),
            ("b", b.clone()),
            ("c", c.into()),
        ])?;
        let stream: LogicalStreamType = Stream::new(
            u,
            PositiveReal::new(1.).unwrap(),
            1,
            Synchronicity::Sync,
            1,
            Direction::Forward,
            None,
            false,
        )
        .into();
        let logical_stream = stream.synthesize();
        assert_eq!(
            logical_stream
                .streams
                .values()
                .map(|physical_stream| physical_stream.dimensionality())
                .collect::<Vec<_>>(),
            vec![1, 1]
        );

        let c = Stream::new(
            LogicalStreamType::Bits(Positive::new(4).unwrap()),
            PositiveReal::new(1.).unwrap(),
            1,
            Synchronicity::Desync,
            1,
            Direction::Forward,
            None,
            false,
        );
        let u = LogicalStreamType::try_new_union(vec![
            ("a", 3.try_into()?),
            ("b", b),
            ("c", c.into()),
        ])?;
        let stream: LogicalStreamType = Stream::new(
            u,
            PositiveReal::new(1.).unwrap(),
            1,
            Synchronicity::Sync,
            1,
            Direction::Forward,
            None,
            false,
        )
        .into();
        let logical_stream = stream.synthesize();
        assert_eq!(
            logical_stream
                .streams
                .values()
                .map(|physical_stream| physical_stream.dimensionality())
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        Ok(())
    }
}
