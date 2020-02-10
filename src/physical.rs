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
    stream::{to_field_name, BitCount, Complexity, Fields, Name},
};
use std::{convert::TryInto, error, num::NonZeroUsize};

/// Element content of a physical stream.
///
/// A field has a name and a bit count.
///
/// - The bit count cannot be zero.
///
/// - The name of each field is a string consisting of letters, numbers, and/or underscores.
/// - The name cannot contain two or more consecutive underscores.
/// - The name cannot start or end with an underscore.
/// - The name cannot start with a digit.
/// - The name cannot be empty.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#element-content
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The name of this field.
    name: String,
    /// The bit count of this field.
    bit_count: NonZeroUsize,
}

impl Field {
    /// Constructs a Field with provided name and bit count. Returns an error
    /// when an invalid name is provided or when the bit count is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{physical::Field, stream::{BitCount, Name}};
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.bit_count(), 8);
    /// assert_eq!(field.name(), "count");
    ///
    /// // Invalid names
    /// assert_eq!(
    ///     Field::new("", 1).unwrap_err().to_string(),
    ///     "Invalid argument: name cannot be empty"
    /// );
    /// assert_eq!(
    ///     Field::new("1count", 1).unwrap_err().to_string(),
    ///     "Invalid argument: name cannot start with a digit"
    /// );
    /// assert_eq!(
    ///     Field::new("_count", 1).unwrap_err().to_string(),
    ///     "Invalid argument: name cannot start or end with an underscore"
    /// );
    /// assert_eq!(
    ///     Field::new("count_", 1).unwrap_err().to_string(),
    ///     "Invalid argument: name cannot start or end with an underscore"
    /// );
    /// //assert_eq!(
    /// //    Field::new("a!@#", 1).unwrap_err().to_string(),
    /// //    "Invalid argument: only letters, numbers and/or underscores are allowed for name"
    /// //);
    ///
    /// // Invalid arguments
    /// assert_eq!(
    ///     Field::new("a", 0).unwrap_err().to_string(),
    ///     "Invalid argument: bit count cannot be zero"
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(name: impl Into<String>, bit_count: usize) -> Result<Self, Box<dyn error::Error>> {
        Ok(Field {
            name: to_field_name(name, false)?,
            bit_count: NonZeroUsize::new(bit_count)
                .ok_or_else(|| Error::InvalidArgument("bit count cannot be zero".to_string()))?,
        })
    }
}

impl Name for Field {
    /// Returns the name of this field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{physical::Field, stream::Name};
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.name(), "count");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl BitCount for Field {
    /// Returns the bit count of this field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::{physical::Field, stream::BitCount};
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.bit_count(), 8);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn bit_count(&self) -> usize {
        self.bit_count.get()
    }
}

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
    fields: Fields<Field>,
    /// Number of element lanes.
    lanes: NonZeroUsize,
    /// Dimensionality.
    dimensionality: usize,
    /// Complexity.
    complexity: Complexity,
    /// User-defined transfer content.
    user: Fields<Field>,
}

/// Returns ⌈log2(x)⌉.
pub(crate) const fn log2_ceil(x: usize) -> usize {
    8 * std::mem::size_of::<usize>() - (x - 1).leading_zeros() as usize
}

impl PhysicalStream {
    /// Constructs a new PhysicalStream using provided arguments. Returns an
    /// error when provided argument are not valid.
    pub fn new(
        fields: impl TryInto<Fields<Field>, Error = Box<dyn error::Error>>,
        lanes: usize,
        dimensionality: usize,
        complexity: impl Into<Complexity>,
        user: impl TryInto<Fields<Field>, Error = Box<dyn error::Error>>,
    ) -> Result<Self, Box<dyn error::Error>> {
        Ok(PhysicalStream {
            fields: fields.try_into()?,
            lanes: NonZeroUsize::new(lanes)
                .ok_or_else(|| Error::InvalidArgument("lanes cannot be zero".to_string()))?,
            dimensionality,
            complexity: complexity.into(),
            user: user.try_into()?,
        })
    }

    /// Returns the fields in this physical stream.
    pub fn fields(&self) -> &[Field] {
        self.fields.as_ref()
    }

    /// Returns the number of element lanes in this physical stream.
    pub fn lanes(&self) -> usize {
        self.lanes.get()
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
    pub fn user(&self) -> &[Field] {
        self.user.as_ref()
    }

    /// Returns the bit count of the data (element) fields in this physical
    /// stream. The bit count is equal to the combined bit count of all fields
    /// multiplied by the number of lanes.
    pub fn data_bit_count(&self) -> usize {
        self.fields.bit_count() * self.lanes.get()
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_bit_count(&self) -> usize {
        self.dimensionality
    }

    /// Returns the number of `stai` (start index) bits in this physical
    /// stream.
    pub fn stai_bit_count(&self) -> usize {
        if self.complexity.major() >= 6 && self.lanes.get() > 1 {
            log2_ceil(self.lanes.get())
        } else {
            0
        }
    }

    /// Returns the number of `endi` (end index) bits in this physical stream.
    pub fn endi_bit_count(&self) -> usize {
        if (self.complexity.major() >= 5 || self.dimensionality >= 1usize) && self.lanes.get() > 1 {
            log2_ceil(self.lanes.get())
        } else {
            0
        }
    }

    /// Returns the number of `strb` (strobe) bits in this physical stream.
    pub fn strb_bit_count(&self) -> usize {
        if self.complexity.major() >= 7 || self.dimensionality >= 1usize {
            self.lanes.get()
        } else {
            0
        }
    }

    /// Returns the bit count of the user fields in this physical stream.
    pub fn user_bit_count(&self) -> usize {
        self.user.bit_count()
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
        self.signal_map().bit_count()
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
    fn field() -> Result<(), Box<dyn error::Error>> {
        let field = Field::new("count", 8)?;
        assert_eq!(field.bit_count(), 8);
        assert_eq!(field.name(), "count");
        assert_eq!(field, field);

        assert_eq!(
            Field::new("", 1).unwrap_err().to_string(),
            "Invalid argument: name cannot be empty"
        );
        assert_eq!(
            Field::new("1count", 1).unwrap_err().to_string(),
            "Invalid argument: name cannot start with a digit"
        );
        assert_eq!(
            Field::new("_count", 1).unwrap_err().to_string(),
            "Invalid argument: name cannot start or end with an underscore"
        );
        assert_eq!(
            Field::new("count_", 1).unwrap_err().to_string(),
            "Invalid argument: name cannot start or end with an underscore"
        );
        assert_eq!(
            Field::new("a!@#", 1).unwrap_err().to_string(),
            "Invalid argument: name must consist of letters, numbers, and/or underscores"
        );
        assert_eq!(
            Field::new("a", 0).unwrap_err().to_string(),
            "Invalid argument: bit count cannot be zero"
        );
        Ok(())
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn physical_stream() -> Result<(), Box<dyn error::Error>> {
        let physical_stream = PhysicalStream::new(
            vec![
                Field::new("a", 8)?,
                Field::new("b", 16)?,
                Field::new("c", 1)?,
            ],
            3,
            4,
            8,
            vec![Field::new("user", 1)?],
        )?;

        assert_eq!(
            physical_stream.fields(),
            &[
                Field::new("a", 8)?,
                Field::new("b", 16)?,
                Field::new("c", 1)?,
            ]
        );
        assert_eq!(physical_stream.lanes(), 3);
        assert_eq!(physical_stream.dimensionality(), 4);
        assert_eq!(physical_stream.complexity(), &Complexity::new_major(8));
        assert_eq!(physical_stream.user(), &[Field::new("user", 1)?]);
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

        let physical_stream = PhysicalStream::new(vec![Field::new("a", 8)?], 1, 0, 0, vec![])?;

        assert_eq!(physical_stream.fields(), &[Field::new("a", 8)?,]);
        assert_eq!(physical_stream.lanes(), 1);
        assert_eq!(physical_stream.dimensionality(), 0);
        assert_eq!(physical_stream.complexity(), &Complexity::new_major(0));
        assert_eq!(physical_stream.user(), &[]);
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
            vec![Field::new("a", 3)?, Field::new("b", 2)?],
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
        Ok(())
    }
}
