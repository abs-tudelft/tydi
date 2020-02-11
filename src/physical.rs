//! Physical streams.
//!
//! This modules defines the components of physical streams as described in the
//! [specification].
//!
//! The [`PhysicalStream`] type wraps the five parameters of a physical stream.
//!
//!
//! [`PhysicalStream`]: ./struct.PhysicalStream.html
//! [specification]: https://abs-tudelft.github.io/tydi/specification/physical.html

use crate::{
    error::Error,
    stream::{BitCount, Complexity},
    util::{log2_ceil, unique_index_map},
};
use indexmap::IndexMap;
use std::{error, num::NonZeroUsize};

/// A physical stream.
///
/// A physical stream carries a stream of elements, dimensionality information
/// for said elements, and (optionally) user-defined transfer information from
/// a source to a sink.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#physical-stream-specification
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalStream {
    /// Element content.
    element: IndexMap<Option<String>, NonZeroUsize>,
    /// Number of element lanes.
    element_lanes: NonZeroUsize,
    /// Dimensionality.
    dimensionality: usize,
    /// Complexity.
    complexity: Complexity,
    /// User-defined transfer content.
    user: IndexMap<Option<String>, NonZeroUsize>,
}

impl PhysicalStream {
    /// Constructs a new PhysicalStream using provided arguments. Returns an
    /// error when provided argument are not valid.
    pub fn new<T, U>(
        element: T,
        element_lanes: usize,
        dimensionality: usize,
        complexity: impl Into<Complexity>,
        user: T,
    ) -> Result<Self, Box<dyn error::Error>>
    where
        T: IntoIterator<Item = (Option<U>, usize)>,
        U: Into<String>,
    {
        let box_err = |msg: &str| Err(Box::new(Error::InvalidArgument(msg.to_string())));

        let validate = |(k, v): (Option<U>, usize)| match (
            k.and_then(|name| {
                let name = name.into();
                if name.is_empty() {
                    None
                } else if name.chars().next().unwrap().is_ascii_digit() {
                    Some(box_err("name cannot start with a digit"))
                } else if name.starts_with('_') || name.ends_with('_') {
                    Some(box_err("name cannot start or end with an underscore"))
                } else if !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c.eq(&'_'))
                {
                    Some(box_err(
                        "name must consist of letters, numbers, and/or underscores",
                    ))
                } else {
                    Some(Ok(name))
                }
            }),
            NonZeroUsize::new(v),
        ) {
            (_, None) => Err(Error::InvalidArgument(
                "bit count cannot be zero".to_string(),
            )),
            (Some(Err(e)), _) => Err(*e),
            (k, Some(v)) => Ok((k.map(|name| name.unwrap()), v)),
        };

        let index_map = |iter: T| {
            unique_index_map(
                iter.into_iter()
                    .map(validate)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Box::new)?,
            )
            .map_err(|_| {
                box_err(
                    "field names must be case-insensitively unique within the set of named fields",
                )
                .unwrap_err()
            })
        };

        Ok(PhysicalStream {
            element: index_map(element)?,
            element_lanes: NonZeroUsize::new(element_lanes).ok_or_else(|| {
                Error::InvalidArgument("element lanes cannot be zero".to_string())
            })?,
            dimensionality,
            complexity: complexity.into(),
            user: index_map(user)?,
        })
    }

    /// Returns the fields in this physical stream.
    pub fn element(&self) -> impl ExactSizeIterator<Item = (&Option<String>, usize)> {
        self.element.iter().map(|(k, v)| (k, v.get()))
    }

    /// Returns the number of element lanes in this physical stream.
    pub fn element_lanes(&self) -> usize {
        self.element_lanes.get()
    }

    /// Returns the dimensionality of this physical stream.
    pub fn dimensionality(&self) -> usize {
        self.dimensionality
    }

    /// Returns the complexity of this physical stream.
    pub fn complexity(&self) -> &Complexity {
        &self.complexity
    }

    /// Returns the user fields in this physical stream.
    pub fn user(&self) -> impl ExactSizeIterator<Item = (&Option<String>, usize)> {
        self.user.iter().map(|(k, v)| (k, v.get()))
    }

    /// Returns the bit count of the data (element) fields in this physical
    /// stream. The bit count is equal to the combined bit count of all fields
    /// multiplied by the number of lanes.
    pub fn data_bit_count(&self) -> usize {
        self.element.values().map(|b| b.get()).sum::<usize>() * self.element_lanes.get()
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_bit_count(&self) -> usize {
        self.dimensionality
    }

    /// Returns the number of `stai` (start index) bits in this physical
    /// stream.
    pub fn stai_bit_count(&self) -> usize {
        if self.complexity.major() >= 6 && self.element_lanes.get() > 1 {
            log2_ceil(self.element_lanes)
        } else {
            0
        }
    }

    /// Returns the number of `endi` (end index) bits in this physical stream.
    pub fn endi_bit_count(&self) -> usize {
        if (self.complexity.major() >= 5 || self.dimensionality >= 1usize)
            && self.element_lanes.get() > 1
        {
            log2_ceil(self.element_lanes)
        } else {
            0
        }
    }

    /// Returns the number of `strb` (strobe) bits in this physical stream.
    pub fn strb_bit_count(&self) -> usize {
        if self.complexity.major() >= 7 || self.dimensionality >= 1usize {
            self.element_lanes.get()
        } else {
            0
        }
    }

    /// Returns the bit count of the user fields in this physical stream.
    pub fn user_bit_count(&self) -> usize {
        self.user.values().map(|b| b.get()).sum::<usize>()
    }

    /// Returns the signal map for this physical stream.
    pub fn signal_map(&self) -> SignalMap {
        let opt = |x| if x == 0 { None } else { Some(x) };
        SignalMap {
            data: opt(self.data_bit_count()),
            last: opt(self.last_bit_count()),
            stai: opt(self.stai_bit_count()),
            endi: opt(self.endi_bit_count()),
            strb: opt(self.strb_bit_count()),
            user: opt(self.user_bit_count()),
        }
    }
}

impl BitCount for PhysicalStream {
    /// Returns the combined bit count of all signals in this physical stream.
    /// This excludes the `valid` and `ready` signals.
    fn bit_count(&self) -> usize {
        self.data_bit_count()
            + self.last_bit_count()
            + self.stai_bit_count()
            + self.endi_bit_count()
            + self.strb_bit_count()
            + self.user_bit_count()
    }
}

impl From<&PhysicalStream> for SignalMap {
    fn from(physical_stream: &PhysicalStream) -> SignalMap {
        physical_stream.signal_map()
    }
}

impl From<PhysicalStream> for SignalMap {
    fn from(physical_stream: PhysicalStream) -> SignalMap {
        physical_stream.signal_map()
    }
}

/// A signal map for a PhysicalStream.
///
/// A signal map can be constructed from a [`PhysicalStream`] using the
/// [`signal_map`] method or using the `From`/`Into` trait implementation.
///
/// [Reference]
///
/// [`PhysicalStream`]: ./struct.PhysicalStream.html
/// [`signal_map`]: ./struct.PhysicalStream.html#method.signal_map
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#signal-omission
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SignalMap {
    data: Option<usize>,
    last: Option<usize>,
    stai: Option<usize>,
    endi: Option<usize>,
    strb: Option<usize>,
    user: Option<usize>,
}

impl SignalMap {
    /// Returns the bit count of the `data` signal.
    pub fn data(&self) -> Option<usize> {
        self.data
    }

    /// Returns the bit count of the `last` signal.
    pub fn last(&self) -> Option<usize> {
        self.last
    }

    /// Returns the bit count of the `stai` signal.
    pub fn stai(&self) -> Option<usize> {
        self.stai
    }

    /// Returns the bit count of the `endi` signal.
    pub fn endi(&self) -> Option<usize> {
        self.endi
    }

    /// Returns the bit count of the `strb` signal.
    pub fn strb(&self) -> Option<usize> {
        self.strb
    }

    /// Returns the bit count of the `user` signal.
    pub fn user(&self) -> Option<usize> {
        self.user
    }

    /// Returns the bit count of all combined signals in this map.
    fn opt_bit_count(&self) -> Option<usize> {
        match self.data.unwrap_or(0)
            + self.last.unwrap_or(0)
            + self.stai.unwrap_or(0)
            + self.endi.unwrap_or(0)
            + self.strb.unwrap_or(0)
            + self.user.unwrap_or(0)
        {
            0 => None,
            x => Some(x),
        }
    }
}

impl<'a> IntoIterator for &'a SignalMap {
    type Item = (&'a str, usize);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            ("data", self.data.unwrap_or(0)),
            ("last", self.last.unwrap_or(0)),
            ("stai", self.stai.unwrap_or(0)),
            ("endi", self.endi.unwrap_or(0)),
            ("strb", self.strb.unwrap_or(0)),
            ("user", self.user.unwrap_or(0)),
        ]
        .into_iter()
    }
}

impl BitCount for SignalMap {
    /// Returns the bit count of all combined signals in this map.
    fn bit_count(&self) -> usize {
        self.opt_bit_count().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn physical_stream() -> Result<(), Box<dyn error::Error>> {
        let physical_stream = PhysicalStream::new(
            vec![(Some("a"), 8), (Some("b"), 16), (Some("c"), 1)],
            3,
            4,
            8,
            vec![(Some("user"), 1)],
        )?;

        let mut element = physical_stream.element();
        assert_eq!(element.next(), Some((&Some("a".to_string()), 8usize)));
        assert_eq!(element.next(), Some((&Some("b".to_string()), 16usize)));
        assert_eq!(element.next(), Some((&Some("c".to_string()), 1usize)));
        assert_eq!(element.next(), None);
        assert_eq!(physical_stream.element_lanes(), 3);
        assert_eq!(physical_stream.dimensionality(), 4);
        assert_eq!(physical_stream.complexity(), &Complexity::new_major(8));
        assert_eq!(
            physical_stream.user().next().unwrap(),
            (&Some("user".to_string()), 1usize)
        );
        assert_eq!(physical_stream.bit_count(), 87);
        assert_eq!(physical_stream.data_bit_count(), (8 + 16 + 1) * 3);
        assert_eq!(physical_stream.last_bit_count(), 4);
        assert_eq!(physical_stream.stai_bit_count(), 2);
        assert_eq!(physical_stream.endi_bit_count(), 2);
        assert_eq!(physical_stream.strb_bit_count(), 3);
        assert_eq!(physical_stream.user_bit_count(), 1);
        assert_eq!(
            physical_stream.signal_map(),
            SignalMap {
                data: Some(75),
                last: Some(4),
                stai: Some(2),
                endi: Some(2),
                strb: Some(3),
                user: Some(1)
            }
        );

        let physical_stream = PhysicalStream::new(vec![(Some("a"), 8)], 1, 0, 0, vec![])?;

        assert_eq!(physical_stream.element().len(), 1);
        assert_eq!(physical_stream.element_lanes(), 1);
        assert_eq!(physical_stream.dimensionality(), 0);
        assert_eq!(physical_stream.complexity(), &Complexity::new_major(0));
        assert_eq!(physical_stream.user().next(), None);
        assert_eq!(physical_stream.bit_count(), 8);
        assert_eq!(physical_stream.data_bit_count(), 8);
        assert_eq!(physical_stream.last_bit_count(), 0);
        assert_eq!(physical_stream.stai_bit_count(), 0);
        assert_eq!(physical_stream.endi_bit_count(), 0);
        assert_eq!(physical_stream.strb_bit_count(), 0);
        assert_eq!(physical_stream.user_bit_count(), 0);
        assert_eq!(
            physical_stream.signal_map(),
            SignalMap {
                data: Some(8),
                last: None,
                stai: None,
                endi: None,
                strb: None,
                user: None
            }
        );

        Ok(())
    }

    #[test]
    fn signal_map() -> Result<(), Box<dyn error::Error>> {
        let physical_stream = PhysicalStream::new(
            vec![(Some("a"), 3), (Some("b"), 2)],
            2,
            3,
            Complexity::new_major(8),
            vec![],
        )?;

        let signal_map = SignalMap::from(&physical_stream);
        assert_eq!(physical_stream.bit_count(), 17);
        assert_eq!(physical_stream.data_bit_count(), 2 * (3 + 2));
        assert_eq!(physical_stream.last_bit_count(), 3);
        assert_eq!(physical_stream.stai_bit_count(), 1);
        assert_eq!(physical_stream.endi_bit_count(), 1);
        assert_eq!(physical_stream.strb_bit_count(), 2);
        assert_eq!(physical_stream.user_bit_count(), 0);

        assert_eq!(
            physical_stream.data_bit_count(),
            signal_map.data().unwrap_or(0)
        );
        assert_eq!(
            physical_stream.last_bit_count(),
            signal_map.last().unwrap_or(0)
        );
        assert_eq!(
            physical_stream.stai_bit_count(),
            signal_map.stai().unwrap_or(0)
        );
        assert_eq!(
            physical_stream.endi_bit_count(),
            signal_map.endi().unwrap_or(0)
        );
        assert_eq!(
            physical_stream.strb_bit_count(),
            signal_map.strb().unwrap_or(0)
        );
        assert_eq!(
            physical_stream.user_bit_count(),
            signal_map.user().unwrap_or(0)
        );

        assert_eq!(signal_map.opt_bit_count(), Some(17));
        assert_eq!(signal_map.bit_count(), 17);
        assert_eq!(signal_map, SignalMap::from(physical_stream));

        assert_eq!(
            signal_map.into_iter().collect::<Vec<_>>(),
            vec![
                ("data", 10),
                ("last", 3),
                ("stai", 1),
                ("endi", 1),
                ("strb", 2),
                ("user", 0)
            ]
        );

        Ok(())
    }
}
