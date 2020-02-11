//! Logical streams.
//!
//! [Reference]
//!
//! [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html

use crate::{
    error::Error,
    stream::{Complexity, Direction},
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

impl Stream {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalStream {
    Null,
    Bits(NonZeroUsize),
    Group(IndexMap<Vec<String>, LogicalStream>),
    Union(IndexMap<Vec<String>, LogicalStream>),
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

    pub fn new_group(
        inner: impl IntoIterator<Item = (Vec<String>, LogicalStream)>,
    ) -> Result<Self, Box<dyn error::Error>> {
        // todo: validation
        Ok(LogicalStream::Group(inner.into_iter().collect()))
    }

    pub fn new_union(
        inner: impl IntoIterator<Item = (Vec<String>, LogicalStream)>,
    ) -> Result<Self, Box<dyn error::Error>> {
        // todo: validation
        Ok(LogicalStream::Union(inner.into_iter().collect()))
    }

    pub fn new_stream(inner: Stream) -> Self {
        LogicalStream::Stream(inner)
    }

    // /// Type compatibility function
    // pub fn compatible(&self, other: &LogicalStream) -> bool {
    //     self == other
    //         || match other {
    //             LogicalStream::Stream {
    //                 data, complexity, ..
    //             } => {
    //                 let (data_, complexity_) = (data, complexity);
    //                 match self {
    //                     LogicalStream::Stream {
    //                         data, complexity, ..
    //                     } => data.compatible(data_) && complexity < complexity_,
    //                     _ => false,
    //                 }
    //             }
    //             _ => false,
    //         }
    //         || match self {
    //             LogicalStream::Group(source) | LogicalStream::Union(source) => match other {
    //                 LogicalStream::Group(sink) | LogicalStream::Union(sink) => {
    //                     source.len() == sink.len()
    //                         && source.iter().zip(sink.iter()).all(|(f, f_)| {
    //                             f.name() == f_.name() && f.stream().compatible(f_.stream())
    //                         })
    //                 }
    //                 _ => false,
    //             },
    //             _ => false,
    //         }
    // }

    // /// Null detection function
    // pub fn is_null(&self) -> bool {
    //     match self {
    //         LogicalStream::Stream {
    //             data, user, keep, ..
    //         } => data.is_null() && user.is_some() && user.as_ref().unwrap().is_null() && !keep,
    //         LogicalStream::Null => true,
    //         LogicalStream::Group(fields) | LogicalStream::Union(fields) => {
    //             fields.iter().all(|f| f.stream().is_null())
    //         }
    //         _ => false,
    //     }
    // }
}

// impl BitCount for LogicalStream {
//     // TODO(mb) check
//     fn bit_count(&self) -> usize {
//         match self {
//             LogicalStream::Null => 0,
//             LogicalStream::Bits(bits) => bits.get(),
//             LogicalStream::Group(fields) => fields.bit_count(),
//             LogicalStream::Union(fields) => {
//                 fields.iter().map(BitCount::bit_count).max().unwrap_or(0)
//             }
//             LogicalStream::Stream { .. } => {
//                 // streams are virtual
//                 // data.bit_count() + user.as_ref().map(|s| s.bit_count()).unwrap_or(0)
//                 0
//             }
//         }
//     }
// }

// trait Split: Sized {
//     fn split(&self) -> (LogicalStream, Fields<Field>);
// }

// impl Split for LogicalStream {
//     fn split(&self) -> (LogicalStream, Fields<Field>) {
//         let t_in = self;
//         match t_in {
//             LogicalStream::Stream {
//                 data,
//                 lanes,
//                 dimensionality,
//                 synchronicity,
//                 complexity,
//                 direction,
//                 user,
//                 keep,
//             } => {
//                 let t_d = data;
//                 let t_u = user;

//                 // Initialize N and T to empty lists.
//                 let mut fields: FieldsBuilder<Field> = FieldsBuilder::new();

//                 let (t_data, extend) = t_d.split();

//                 if !t_data.is_null() || t_u.is_some() && !t_u.as_ref().unwrap().is_null() || *keep {
//                     fields.add_field(
//                         Field::new(
//                             None as Option<&str>,
//                             LogicalStream::stream(
//                                 t_data,
//                                 lanes.get(),
//                                 *dimensionality,
//                                 *synchronicity,
//                                 complexity.clone(),
//                                 *direction,
//                                 t_u.clone(),
//                                 *keep,
//                             )
//                             .unwrap(),
//                         )
//                         .unwrap(),
//                     );
//                 }

//                 // append names and streams.
//                 fields.extend(extend.into_iter().map(|named_logical_stream| {
//                     match named_logical_stream.stream {
//                         LogicalStream::Stream {
//                             data,
//                             lanes,
//                             dimensionality,
//                             synchronicity,
//                             complexity,
//                             direction,
//                             user,
//                             keep,
//                         } => {
//                             let direction = if t_in.direction() == Direction::Reverse {
//                                 t_in.direction().reversed()
//                             } else {
//                                 direction
//                             };
//                             let synchronicity = if t_in.synchronicity() == Synchronicity::Flatten
//                                 || t_in.synchronicity() == Synchronicity::FlatDesync
//                             {
//                                 Synchronicity::FlatDesync
//                             } else {
//                                 synchronicity
//                             };
//                             let dimensionality = if synchronicity != Synchronicity::Flatten
//                                 && t_in.synchronicity() != Synchronicity::FlatDesync
//                             {
//                                 dimensionality + t_in.dimensionality()
//                             } else {
//                                 dimensionality
//                             };
//                             let lanes = lanes.get() * t_in.lanes();
//                             NamedLogicalStream::new(
//                                 named_logical_stream.name,
//                                 LogicalStream::stream(
//                                     *data,
//                                     lanes,
//                                     dimensionality,
//                                     synchronicity,
//                                     complexity,
//                                     direction,
//                                     user,
//                                     keep,
//                                 )
//                                 .unwrap(),
//                             )
//                             .unwrap()
//                         }
//                         _ => unreachable!(),
//                     }
//                 }));
//                 (LogicalStream::Null, fields.finish().unwrap())
//             }
//             LogicalStream::Null | LogicalStream::Bits(_) => {
//                 (t_in.clone(), Fields::new(vec![]).unwrap())
//             }
//             LogicalStream::Group(inner) | LogicalStream::Union(inner) => {
//                 let mut fields: FieldsBuilder<Field> = FieldsBuilder::new();
//                 fields.extend(inner.into_iter().map(|named_logical_stream| {
//                     Field::new(
//                         named_logical_stream.name.to_string(),
//                         named_logical_stream.stream.split().0,
//                     )
//                     .unwrap()
//                 }));
//                 let fields = fields.finish().unwrap();
//                 let t_signals = if t_in.is_group() {
//                     LogicalStream::Group(fields)
//                 } else {
//                     LogicalStream::Union(fields)
//                 };

//                 let mut fields: FieldsBuilder<Field> = FieldsBuilder::new();
//                 inner.into_iter().for_each(|named_logical_stream| {
//                     fields.extend(
//                         named_logical_stream
//                             .stream
//                             .split()
//                             .1
//                             .into_iter()
//                             .map(|inner| {
//                                 let name = match inner.name {
//                                     Some(name) => {
//                                         format!("{}__{}", named_logical_stream.name(), name)
//                                     }
//                                     None => named_logical_stream.name().to_string(),
//                                 };
//                                 Field::new(Some(name), inner.stream).unwrap()
//                             }),
//                     );
//                 });

//                 (t_signals, fields.finish().unwrap())
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::error;

//     #[test]
//     fn split() -> Result<(), Box<dyn error::Error>> {
//         let logical_stream = LogicalStream::stream(
//             LogicalStream::group(vec![
//                 Field::new("a", LogicalStream::bits(4)?)?,
//                 Field::new("b", LogicalStream::bits(4)?)?,
//                 Field::new(
//                     "c",
//                     LogicalStream::stream(
//                         LogicalStream::union(vec![
//                             Field::new("a", LogicalStream::bits(4)?)?,
//                             Field::new("b", LogicalStream::bits(4)?)?,
//                             Field::new(
//                                 "d",
//                                 LogicalStream::stream(
//                                     LogicalStream::union(vec![
//                                         Field::new("e", LogicalStream::bits(4)?)?,
//                                         Field::new("f", LogicalStream::bits(4)?)?,
//                                     ])?,
//                                     1,
//                                     1,
//                                     Synchronicity::Desync,
//                                     0,
//                                     Direction::Reverse,
//                                     None,
//                                     false,
//                                 )?,
//                             )?,
//                         ])?,
//                         2,
//                         1,
//                         Synchronicity::Desync,
//                         0,
//                         Direction::Reverse,
//                         None,
//                         false,
//                     )?,
//                 )?,
//             ])?,
//             1,
//             0,
//             Synchronicity::Desync,
//             0,
//             Direction::Forward,
//             None,
//             false,
//         )?;

//         let _ = logical_stream.split();
//         Ok(())
//     }
// }
