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

use crate::error::Error;
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    error, fmt,
    iter::FromIterator,
    ops::Deref,
    slice::Iter,
    str::FromStr,
};

/// Type safe bit counts.
///
/// The bit count cannot be zero. This newtype prevents that.
///
/// # Examples
///
/// ```rust
/// use std::convert::TryFrom;
/// use tydi::physical::BitCount;
///
/// let bit_count = BitCount::new(7)?;
/// assert_eq!(bit_count.bit_count(), 7);
/// assert_eq!(bit_count, BitCount::try_from(7)?);
///
/// assert_eq!(
///     BitCount::new(0).unwrap_err().to_string(),
///     "Invalid argument: count cannot be zero"
/// );
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#bit-count
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct BitCount(usize);

impl BitCount {
    /// Returns a new BitCount with provided count as bit count. Returns an
    /// error when the count is zero.
    pub fn new(count: usize) -> Result<Self, Box<dyn error::Error>> {
        if count == 0 {
            Err(Box::new(Error::InvalidArgument(
                "count cannot be zero".to_string(),
            )))
        } else {
            Ok(BitCount(count))
        }
    }

    /// Returns the bit count.
    pub fn bit_count(self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for BitCount {
    type Error = Box<dyn error::Error>;

    /// Returns a new BitCount when the provided count is greater than zero.
    fn try_from(count: usize) -> Result<Self, Self::Error> {
        BitCount::new(count)
    }
}

/// Logical stream interface complexity level.
///
/// This logical stream parameter specifies the guarantees a source makes about
/// how elements are transferred. Equivalently, it specifies the assumptions a
/// sink can safely make.
///
/// # Examples
///
/// ```rust
/// use tydi::physical::Complexity;
///
/// let c3 = Complexity::new_major(3);
/// let c30 = Complexity::new(vec![3, 0])?;
/// let c31 = Complexity::new(vec![3, 1])?;
/// let c4 = Complexity::new_major(4);
///
/// assert_eq!(c3, c30);
/// assert!(c3 < c31);
/// assert!(c31 < c4);
///
/// assert_eq!(c31.to_string(), "3.1");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#complexity-c
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct Complexity {
    /// The complexity level.
    level: Vec<usize>,
}

impl PartialEq for Complexity {
    /// A complexity number is higher than another when the leftmost integer is
    /// greater, and lower when the leftmost integer is lower. If the leftmost
    /// integer is equal, the next integer is checked recursively. If one
    /// complexity number has more entries than another, the shorter number is
    /// padded with zeros on the right.
    fn eq(&self, other: &Self) -> bool {
        (0..self.level.len().max(other.level.len()))
            .all(|idx| self.level.get(idx).unwrap_or(&0) == other.level.get(idx).unwrap_or(&0))
    }
}

impl Eq for Complexity {}

impl From<usize> for Complexity {
    /// Convert a usize into complexity with the usize as major version.
    fn from(major: usize) -> Self {
        Complexity::new_major(major)
    }
}

impl TryFrom<Vec<usize>> for Complexity {
    type Error = Box<dyn error::Error>;
    /// Try to convert a vector of usize into a complexity. Returns an error
    /// when the provided vector is empty.
    fn try_from(level: Vec<usize>) -> Result<Self, Self::Error> {
        Complexity::new(level)
    }
}

impl Complexity {
    /// Constructs a new Complexity with provided level. Returns an error when
    /// the provided level iterator is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![1, 2, 3, 4])?;
    /// assert!(Complexity::new(vec![]).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(level: impl IntoIterator<Item = usize>) -> Result<Self, Box<dyn error::Error>> {
        let level = level.into_iter().collect::<Vec<usize>>();
        if level.is_empty() {
            Err(Box::new(Error::InvalidArgument(
                "complexity level cannot be empty".to_string(),
            )))
        } else {
            Ok(Complexity { level })
        }
    }

    /// Constructs a new Complexity with provided level as major version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new_major(4);
    ///
    /// assert_eq!(c, Complexity::new(vec![4])?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new_major(level: usize) -> Self {
        Complexity { level: vec![level] }
    }

    /// Returns the level of this Complexity.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.level(), &[3, 14]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn level(&self) -> &[usize] {
        self.level.as_ref()
    }

    /// Returns the major version of this Complexity level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.major(), 3);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    pub fn major(&self) -> usize {
        self.level[0]
    }
}

impl fmt::Display for Complexity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let mut level = self.level.iter();
        if let Some(x) = level.next() {
            result.push_str(&x.to_string());

            for x in level {
                result.push('.');
                result.push_str(&x.to_string());
            }
        }
        write!(f, "{}", result)
    }
}

/// Type safe field names.
///
/// Newtype for valid name used as [`Field`] names.
/// - The name of each field is a string consisting of letters, numbers, and/or underscores.
/// - The name cannot start or end with an underscore.
/// - The name cannot start with a digit.
/// - The name cannot be empty.
///
/// [Reference]
///
/// [`Field`]: ./struct.Field.html
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#field-name
#[derive(Debug, Clone, PartialEq)]
pub struct FieldName(String);

impl FieldName {
    /// Returns a new valid FieldName if the provided name is valid.
    pub fn new(name: impl Into<String>) -> Result<Self, Box<dyn error::Error>> {
        let name = name.into();
        if name.is_empty() {
            Err(Box::new(Error::InvalidArgument(
                "cannot be empty".to_string(),
            )))
        } else if name.chars().next().unwrap().is_ascii_digit() {
            Err(Box::new(Error::InvalidArgument(
                "cannot start with a digit".to_string(),
            )))
        } else if name.starts_with('_') | name.ends_with('_') {
            Err(Box::new(Error::InvalidArgument(
                "cannot start or end with an underscore".to_string(),
            )))
        } else if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            Err(Box::new(Error::InvalidArgument(
                "only letters, numbers and/or underscores are allowed".to_string(),
            )))
        } else {
            Ok(FieldName(name))
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for FieldName {
    type Err = Box<dyn error::Error>;

    /// Returns a new valid FieldName if the provided name is valid.
    fn from_str(name: &str) -> Result<Self, Self::Err> {
        FieldName::new(name)
    }
}

impl TryFrom<String> for FieldName {
    type Error = Box<dyn error::Error>;

    /// Returns a new valid FieldName if the provided name is valid.
    fn try_from(name: String) -> Result<Self, Self::Error> {
        FieldName::new(name)
    }
}

impl TryFrom<&str> for FieldName {
    type Error = Box<dyn error::Error>;

    /// Returns a new valid FieldName if the provided name is valid.
    fn try_from(name: &str) -> Result<Self, Self::Error> {
        FieldName::new(name)
    }
}

impl AsRef<str> for FieldName {
    /// Returns a string slice to the name.
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Type safe lanes parameter.
///
/// Newtype for number of element lanes parameter of a [`PhysicalStream`].
/// Lanes must be an integer greater than or equal to one.
///
/// # Examples
///
/// ```rust
/// use std::convert::TryFrom;
/// use tydi::physical::Lanes;
///
/// let lanes = Lanes::new(7)?;
/// assert_eq!(lanes.lanes(), 7);
/// assert_eq!(lanes, Lanes::try_from(7)?);
///
/// assert_eq!(
///     Lanes::new(0).unwrap_err().to_string(),
///     "Invalid argument: lanes must be greater than or equal to one"
/// );
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [Reference]
///
/// [`PhysicalStream`]: ./struct.PhysicalStream.html
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#number-of-element-lanes-n
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lanes(usize);

impl Lanes {
    /// Returns a new Lanes if provided lanes is greater than or equal to one.
    pub fn new(lanes: usize) -> Result<Self, Box<dyn error::Error>> {
        if lanes < 1 {
            Err(Box::new(Error::InvalidArgument(
                "lanes must be greater than or equal to one".to_string(),
            )))
        } else {
            Ok(Lanes(lanes))
        }
    }

    /// Returns the number of element lanes.
    pub fn lanes(self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for Lanes {
    type Error = Box<dyn error::Error>;
    /// Returns a new Lanes if provided lanes is greater than or equal to one.
    fn try_from(lanes: usize) -> Result<Self, Self::Error> {
        Lanes::new(lanes)
    }
}

/// Type safe fields container.
///
/// Names of fields must be case-insensitively unique within this set of named
/// fields. This newtype validates this condition.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#field-name
#[derive(Debug, Clone, PartialEq)]
pub struct Fields(Vec<Field>);

impl Fields {
    /// Returns a new Fields container with provided Fields. Returns an error
    /// when the names of the provided fields are not case-insensitively
    /// unique.
    pub fn new(fields: impl IntoIterator<Item = Field>) -> Result<Self, Box<dyn error::Error>> {
        let fields: Vec<Field> = fields.into_iter().collect();
        if HashSet::<String>::from_iter(fields.iter().map(|f| f.name().to_ascii_lowercase())).len()
            != fields.len()
        {
            Err(Box::new(Error::InvalidArgument(
                "field names must be case-insensitively unique within the set of named fields"
                    .to_string(),
            )))
        } else {
            Ok(Fields(fields))
        }
    }

    /// Returns the fields in this container.
    pub fn fields(&self) -> &[Field] {
        self.0.as_ref()
    }

    /// Returns the number of fields in this container.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no fields in this container.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the combined bit count of the fields in this container.
    pub fn bit_count(&self) -> usize {
        self.0.iter().map(Field::bit_count).sum()
    }
}

impl<'a> IntoIterator for &'a Fields {
    type Item = &'a Field;
    type IntoIter = Iter<'a, Field>;

    /// Returns an iterator over the fields in this Fields container.
    fn into_iter(self) -> Self::IntoIter {
        self.fields().iter()
    }
}

impl Deref for Fields {
    type Target = [Field];

    /// Deref to a slice of the fields in this Fields container.
    fn deref(&self) -> &[Field] {
        self.fields()
    }
}

impl TryFrom<Vec<Field>> for Fields {
    type Error = Box<dyn error::Error>;

    /// Returns a new Fields container with provided Fields. Returns an error
    /// when the names of the provided fields are not case-insensitively
    /// unique.
    fn try_from(fields: Vec<Field>) -> Result<Self, Self::Error> {
        Fields::new(fields)
    }
}

/// Element content of a physical stream.
///
/// A field has a name and a bit count.
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#element-content
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The name of this field.
    name: FieldName,
    /// The bit count of this field.
    bit_count: BitCount,
}

impl Field {
    /// Constructs a Field with provided name and bit count. Returns an error
    /// when an invalid name is provided or when the bit count is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Field;
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.bit_count(), 8);
    /// assert_eq!(field.name(), "count");
    ///
    /// // Invalid names
    /// assert_eq!(
    ///     Field::new("", 1).unwrap_err().to_string(),
    ///     "Invalid argument: cannot be empty"
    /// );
    /// assert_eq!(
    ///     Field::new("1count", 1).unwrap_err().to_string(),
    ///     "Invalid argument: cannot start with a digit"
    /// );
    /// assert_eq!(
    ///     Field::new("_count", 1).unwrap_err().to_string(),
    ///     "Invalid argument: cannot start or end with an underscore"
    /// );
    /// assert_eq!(
    ///     Field::new("count_", 1).unwrap_err().to_string(),
    ///     "Invalid argument: cannot start or end with an underscore"
    /// );
    /// assert_eq!(
    ///     Field::new("a!@#", 1).unwrap_err().to_string(),
    ///     "Invalid argument: only letters, numbers and/or underscores are allowed"
    /// );
    ///
    /// // Invalid arguments
    /// assert_eq!(
    ///     Field::new("a", 0).unwrap_err().to_string(),
    ///     "Invalid argument: count cannot be zero"
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        name: impl TryInto<FieldName, Error = Box<dyn error::Error>>,
        bit_count: impl TryInto<BitCount, Error = Box<dyn error::Error>>,
    ) -> Result<Self, Box<dyn error::Error>> {
        Ok(Field {
            name: name.try_into()?,
            bit_count: bit_count.try_into()?,
        })
    }

    /// Returns the name of this field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Field;
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.name(), "count");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the bit count of this field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Field;
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.bit_count(), 8);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn bit_count(&self) -> usize {
        self.bit_count.bit_count()
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
    fields: Fields,
    /// Number of element lanes.
    lanes: Lanes,
    /// Dimensionality.
    dimensionality: usize,
    /// Complexity.
    complexity: Complexity,
    /// User-defined transfer content.
    user: Fields,
}

impl PhysicalStream {
    /// Constructs a new PhysicalStream using provided arguments. Returns an
    /// error when provided argument are not valid.
    pub fn new(
        fields: impl TryInto<Fields, Error = Box<dyn error::Error>>,
        lanes: impl TryInto<Lanes, Error = Box<dyn error::Error>>,
        dimensionality: usize,
        complexity: impl Into<Complexity>,
        user: impl TryInto<Fields, Error = Box<dyn error::Error>>,
    ) -> Result<Self, Box<dyn error::Error>> {
        Ok(PhysicalStream {
            fields: fields.try_into()?,
            lanes: lanes.try_into()?,
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
        self.lanes.lanes()
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

    /// Returns the combined bit count of all signals in this physical stream.
    /// This excludes the `valid` and `ready` signals.
    pub fn bit_count(&self) -> usize {
        self.signal_map().bit_count().unwrap_or(0)
    }

    /// Returns the bit count of the data (element) fields in this physical
    /// stream. The bit count is equal to the combined bit count of all fields
    /// multiplied by the number of lanes.
    pub fn data_bit_count(&self) -> usize {
        self.fields.bit_count() * self.lanes.lanes()
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_bit_count(&self) -> usize {
        self.dimensionality
    }

    /// Returns the number of `stai` (start index) bits in this physical
    /// stream.
    pub fn stai_bit_count(&self) -> usize {
        if self.complexity.major() >= 6 && self.lanes.lanes() > 1 {
            // ⌈log2N⌉
            // TODO(jvstraten): fix this
            (self.lanes.lanes() as f64).log2().ceil() as usize
        } else {
            0
        }
    }

    /// Returns the number of `endi` (end index) bits in this physical stream.
    pub fn endi_bit_count(&self) -> usize {
        if (self.complexity.major() >= 5 || self.dimensionality >= 1) && self.lanes.lanes() > 1 {
            // ⌈log2N⌉
            // TODO(jvstraten): fix this
            (self.lanes.lanes() as f64).log2().ceil() as usize
        } else {
            0
        }
    }

    /// Returns the number of `strb` (strobe) bits in this physical stream.
    pub fn strb_bit_count(&self) -> usize {
        if self.complexity.major() >= 7 || self.dimensionality >= 1 {
            self.lanes.lanes()
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
    pub fn bit_count(&self) -> Option<usize> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    #[test]
    fn bit_count() -> Result<(), Box<dyn std::error::Error>> {
        let bit_count = BitCount::new(7)?;
        assert_eq!(bit_count.bit_count(), 7);
        assert_eq!(bit_count, BitCount::try_from(7)?);

        assert_eq!(
            BitCount::new(0).unwrap_err().to_string(),
            "Invalid argument: count cannot be zero"
        );
        Ok(())
    }

    #[test]
    fn complexity() -> Result<(), Box<dyn error::Error>> {
        let empty = Complexity::new(vec![]);
        assert_eq!(
            empty.unwrap_err().to_string(),
            "Invalid argument: complexity level cannot be empty"
        );
        assert_eq!(
            Complexity::try_from(vec![]).unwrap_err().to_string(),
            "Invalid argument: complexity level cannot be empty"
        );

        let c = Complexity::new_major(0);
        let c3 = Complexity::new_major(3);
        let c30 = Complexity::new(vec![3, 0])?;
        let c31 = Complexity::new(vec![3, 1])?;
        let c311 = Complexity::new(vec![3, 1, 1])?;
        let c32 = Complexity::new(vec![3, 2])?;
        let c4 = Complexity::new_major(4);
        let c400 = Complexity::new(vec![4, 0, 0])?;
        let c401 = Complexity::new(vec![4, 0, 1])?;
        assert!(c < c3);
        assert!(c3 < c31);
        assert_eq!(c3, c30);
        assert!(c31 < c311);
        assert!(c311 < c32);
        assert!(c32 < c4);
        assert_eq!(c4, c4);
        assert_eq!(c4, c400);
        assert_eq!(c400, c4);
        assert!(c400 < c401);
        assert!(c4 < c401);
        assert_eq!(c3, 3.into());
        assert_eq!(c401, vec![4, 0, 1].try_into()?);

        assert_eq!(c3.to_string(), "3");
        assert_eq!(c31.to_string(), "3.1");

        assert_eq!(c3.major(), 3);
        assert_eq!(c31.major(), 3);
        assert_eq!(c4.major(), 4);

        assert_eq!(c4.level(), &[4]);
        assert_eq!(c400.level(), &[4, 0, 0]);
        Ok(())
    }

    #[test]
    fn field() -> Result<(), Box<dyn error::Error>> {
        let field = Field::new("count", 8)?;
        assert_eq!(field.bit_count(), 8);
        assert_eq!(field.name(), "count");
        assert_eq!(field, field);

        assert_eq!(
            Field::new("", 1).unwrap_err().to_string(),
            "Invalid argument: cannot be empty"
        );
        assert_eq!(
            Field::new("1count", 1).unwrap_err().to_string(),
            "Invalid argument: cannot start with a digit"
        );
        assert_eq!(
            Field::new("_count", 1).unwrap_err().to_string(),
            "Invalid argument: cannot start or end with an underscore"
        );
        assert_eq!(
            Field::new("count_", 1).unwrap_err().to_string(),
            "Invalid argument: cannot start or end with an underscore"
        );
        assert_eq!(
            Field::new("a!@#", 1).unwrap_err().to_string(),
            "Invalid argument: only letters, numbers and/or underscores are allowed"
        );
        assert_eq!(
            Field::new("a", 0).unwrap_err().to_string(),
            "Invalid argument: count cannot be zero"
        );
        Ok(())
    }

    #[test]
    fn fields() -> Result<(), Box<dyn error::Error>> {
        let fields = Fields::new(vec![])?;
        assert_eq!(fields.len(), 0);
        assert!(fields.is_empty());

        let fields = Fields::new(vec![Field::new("a", 3)?])?;
        assert_eq!(fields.len(), 1);
        assert_eq!(fields.bit_count(), 3);
        assert_eq!(Fields::try_from(vec![Field::new("a", 3)?])?, fields);

        let fields = Fields::new(vec![Field::new("a", 3)?, Field::new("b", 5)?])?;
        assert_eq!(fields.len(), 2);
        assert_eq!(fields.bit_count(), 8);
        assert_eq!(fields.iter().len(), 2);
        assert_eq!(fields.fields(), &[Field::new("a", 3)?, Field::new("b", 5)?]);
        assert_eq!(
            fields
                .into_iter()
                .map(|field| field.bit_count())
                .sum::<usize>(),
            fields.bit_count()
        );

        let fields = Fields::new(vec![Field::new("a", 3)?, Field::new("a", 3)?]);
        assert_eq!(
            fields.unwrap_err().to_string(),
            "Invalid argument: field names must be case-insensitively unique within the set of named fields"
        );

        Ok(())
    }

    #[test]
    fn field_name() -> Result<(), Box<dyn error::Error>> {
        assert_eq!("asdf", FieldName::from_str("asdf")?.as_ref());
        assert_eq!(FieldName::from_str("asdf")?.name(), "asdf");
        assert_eq!(
            FieldName::try_from("asdf".to_string())?,
            FieldName::try_from("asdf")?
        );
        let field_name = FieldName::new("asdf")?;
        assert_eq!(field_name.to_string(), "asdf");
        assert_eq!(field_name, field_name);

        assert_eq!(
            FieldName::new("",).unwrap_err().to_string(),
            "Invalid argument: cannot be empty"
        );
        assert_eq!(
            FieldName::new("1count").unwrap_err().to_string(),
            "Invalid argument: cannot start with a digit"
        );
        assert_eq!(
            FieldName::new("_count").unwrap_err().to_string(),
            "Invalid argument: cannot start or end with an underscore"
        );
        assert_eq!(
            FieldName::new("count_").unwrap_err().to_string(),
            "Invalid argument: cannot start or end with an underscore"
        );
        assert_eq!(
            FieldName::new("a!@#").unwrap_err().to_string(),
            "Invalid argument: only letters, numbers and/or underscores are allowed"
        );

        Ok(())
    }

    #[test]
    fn lanes() -> Result<(), Box<dyn error::Error>> {
        let lanes = Lanes::new(7)?;
        assert_eq!(lanes.lanes(), 7);
        assert_eq!(lanes, Lanes::try_from(7)?);
        assert!(Lanes::new(6)? < lanes);

        assert_eq!(
            Lanes::new(0).unwrap_err().to_string(),
            "Invalid argument: lanes must be greater than or equal to one"
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

        assert_eq!(signal_map.bit_count(), Some(17));
        assert_eq!(signal_map, SignalMap::from(physical_stream));
        Ok(())
    }
}
