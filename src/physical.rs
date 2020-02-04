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
/// assert_eq!(bit_count.width(), 7);
/// assert_eq!(bit_count, BitCount::try_from(7)?);
///
/// assert_eq!(
///     BitCount::new(0).unwrap_err().to_string(),
///     "Invalid argument: width cannot be zero"
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
    /// Returns a new BitCount with provided width as bit count. Returns an
    /// error when the width is zero.
    pub fn new(width: usize) -> Result<Self, Box<dyn error::Error>> {
        if width == 0 {
            Err(Box::new(Error::InvalidArgument(
                "width cannot be zero".to_string(),
            )))
        } else {
            Ok(BitCount(width))
        }
    }

    /// Returns the width (bit count).
    pub fn width(self) -> usize {
        self.0
    }
}

impl TryFrom<usize> for BitCount {
    type Error = Box<dyn error::Error>;

    /// Returns a new BitCount when the provided width is greater than zero.
    fn try_from(width: usize) -> Result<Self, Self::Error> {
        BitCount::new(width)
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

/// Type safe identifiers.
///
/// Newtype for valid identifiers used as [`Field`] names.
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
pub struct Identifier(String);

impl Identifier {
    /// Returns a new valid Identifier if the provided identifier is valid.
    pub fn new(identifier: impl Into<String>) -> Result<Self, Box<dyn error::Error>> {
        let identifier = identifier.into();
        if identifier.is_empty() {
            Err(Box::new(Error::InvalidIdentifier(
                "cannot be empty".to_string(),
            )))
        } else if identifier.chars().next().unwrap().is_ascii_digit() {
            Err(Box::new(Error::InvalidIdentifier(
                "cannot start with a digit".to_string(),
            )))
        } else if identifier.starts_with('_') | identifier.ends_with('_') {
            Err(Box::new(Error::InvalidIdentifier(
                "cannot start or end with an underscore".to_string(),
            )))
        } else if !identifier.chars().all(|c| c.is_ascii_alphanumeric()) {
            Err(Box::new(Error::InvalidIdentifier(
                "only letters, numbers and/or underscores are allowed".to_string(),
            )))
        } else {
            Ok(Identifier(identifier))
        }
    }
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Identifier {
    type Err = Box<dyn error::Error>;

    /// Returns a new valid Identifier if the provided identifier is valid.
    fn from_str(identifier: &str) -> Result<Self, Self::Err> {
        Identifier::new(identifier)
    }
}

impl TryFrom<String> for Identifier {
    type Error = Box<dyn error::Error>;

    /// Returns a new valid Identifier if the provided identifier is valid.
    fn try_from(identifier: String) -> Result<Self, Self::Error> {
        Identifier::new(identifier)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = Box<dyn error::Error>;

    /// Returns a new valid Identifier if the provided identifier is valid.
    fn try_from(identifier: &str) -> Result<Self, Self::Error> {
        Identifier::new(identifier)
    }
}

impl AsRef<str> for Identifier {
    /// Returns a string slice to the identifier.
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
        if HashSet::<String>::from_iter(fields.iter().map(|f| f.identifier().to_ascii_lowercase()))
            .len()
            != fields.len()
        {
            Err(Box::new(Error::InvalidArgument(
                    "field identifiers must be case-insensitively unique within the set of named fields"
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

    /// Returns the combined width of the fields in this container.
    pub fn width(&self) -> usize {
        self.0.iter().map(Field::width).sum()
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
/// A field has an identifier (name) and a width (bit count).
///
/// [Reference]
///
/// [Reference]: https://abs-tudelft.github.io/tydi/specification/physical.html#element-content
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The name of this field.
    identifier: Identifier,
    /// The bit count of this field.
    width: BitCount,
}

impl Field {
    /// Constructs a Field with provided identifier as name and width as bit
    /// count. Returns an error when an invalid identifier is provided or when
    /// the bit count is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Field;
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.width(), 8);
    /// assert_eq!(field.identifier(), "count");
    ///
    /// // Invalid identifiers
    /// assert_eq!(
    ///     Field::new("", 1).unwrap_err().to_string(),
    ///     "Invalid identifier: cannot be empty"
    /// );
    /// assert_eq!(
    ///     Field::new("1count", 1).unwrap_err().to_string(),
    ///     "Invalid identifier: cannot start with a digit"
    /// );
    /// assert_eq!(
    ///     Field::new("_count", 1).unwrap_err().to_string(),
    ///     "Invalid identifier: cannot start or end with an underscore"
    /// );
    /// assert_eq!(
    ///     Field::new("count_", 1).unwrap_err().to_string(),
    ///     "Invalid identifier: cannot start or end with an underscore"
    /// );
    /// assert_eq!(
    ///     Field::new("a!@#", 1).unwrap_err().to_string(),
    ///     "Invalid identifier: only letters, numbers and/or underscores are allowed"
    /// );
    ///
    /// // Invalid arguments
    /// assert_eq!(
    ///     Field::new("count", 0).unwrap_err().to_string(),
    ///     "Invalid argument: width cannot be zero"
    /// );
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        identifier: impl TryInto<Identifier, Error = Box<dyn error::Error>>,
        width: impl TryInto<BitCount, Error = Box<dyn error::Error>>,
    ) -> Result<Self, Box<dyn error::Error>> {
        Ok(Field {
            identifier: identifier.try_into()?,
            width: width.try_into()?,
        })
    }

    /// Returns the identifier (name) of this field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Field;
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.identifier(), "count");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn identifier(&self) -> &str {
        self.identifier.as_ref()
    }

    /// Returns the width (bit count) of this field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::physical::Field;
    ///
    /// let field = Field::new("count", 8)?;
    /// assert_eq!(field.width(), 8);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn width(&self) -> usize {
        self.width.width()
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

    /// Returns the combined width of all signals in this physical stream. This
    /// excludes the `valid` and `ready` signals.
    pub fn width(&self) -> usize {
        self.signal_map().width().unwrap_or(0)
    }

    /// Returns the width of the data (element) fields in this physical stream.
    /// The width is equal to the combined width of all fields multiplied by
    /// the number of lanes.
    pub fn data_width(&self) -> usize {
        self.fields.width() * self.lanes.lanes()
    }

    /// Returns the number of last bits in this physical stream. The number of
    /// last bits equals the dimensionality.
    pub fn last_width(&self) -> usize {
        self.dimensionality
    }

    /// Returns the number of `stai` (start index) bits in this physical
    /// stream.
    pub fn stai_width(&self) -> usize {
        if self.complexity.major() >= 6 && self.lanes.lanes() > 1 {
            // ⌈log2N⌉
            // TODO(jvstraten): fix this
            (self.lanes.lanes() as f64).log2().ceil() as usize
        } else {
            0
        }
    }

    /// Returns the number of `endi` (end index) bits in this physical stream.
    pub fn endi_width(&self) -> usize {
        if (self.complexity.major() >= 5 || self.dimensionality >= 1) && self.lanes.lanes() > 1 {
            // ⌈log2N⌉
            // TODO(jvstraten): fix this
            (self.lanes.lanes() as f64).log2().ceil() as usize
        } else {
            0
        }
    }

    /// Returns the number of `strb` (strobe) bits in this physical stream.
    pub fn strb_width(&self) -> usize {
        if self.complexity.major() >= 7 || self.dimensionality >= 1 {
            self.lanes.lanes()
        } else {
            0
        }
    }

    /// Returns the width of the user fields in this physical stream.
    pub fn user_width(&self) -> usize {
        self.user.width()
    }

    /// Returns the signal map for this physical stream.
    pub fn signal_map(&self) -> SignalMap {
        let opt = |x| if x == 0 { None } else { Some(x) };
        SignalMap {
            data: opt(self.data_width()),
            last: opt(self.last_width()),
            stai: opt(self.stai_width()),
            endi: opt(self.endi_width()),
            strb: opt(self.strb_width()),
            user: opt(self.user_width()),
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
pub struct SignalMap {
    data: Option<usize>,
    last: Option<usize>,
    stai: Option<usize>,
    endi: Option<usize>,
    strb: Option<usize>,
    user: Option<usize>,
}

impl SignalMap {
    /// Returns the width of the `data` signal.
    pub fn data(&self) -> Option<usize> {
        self.data
    }

    /// Returns the width of the `last` signal.
    pub fn last(&self) -> Option<usize> {
        self.last
    }

    /// Returns the width of the `stai` signal.
    pub fn stai(&self) -> Option<usize> {
        self.stai
    }

    /// Returns the width of the `endi` signal.
    pub fn endi(&self) -> Option<usize> {
        self.endi
    }

    /// Returns the width of the `strb` signal.
    pub fn strb(&self) -> Option<usize> {
        self.strb
    }

    /// Returns the width of the `user` signal.
    pub fn user(&self) -> Option<usize> {
        self.user
    }

    /// Returns the width of all combined signals in this map.
    pub fn width(&self) -> Option<usize> {
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
        assert_eq!(bit_count.width(), 7);
        assert_eq!(bit_count, BitCount::try_from(7)?);

        assert_eq!(
            BitCount::new(0).unwrap_err().to_string(),
            "Invalid argument: width cannot be zero"
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
        assert_eq!(field.width(), 8);
        assert_eq!(field.identifier(), "count");
        assert_eq!(field, field);

        assert_eq!(
            Field::new("", 1).unwrap_err().to_string(),
            "Invalid identifier: cannot be empty"
        );
        assert_eq!(
            Field::new("1count", 1).unwrap_err().to_string(),
            "Invalid identifier: cannot start with a digit"
        );
        assert_eq!(
            Field::new("_count", 1).unwrap_err().to_string(),
            "Invalid identifier: cannot start or end with an underscore"
        );
        assert_eq!(
            Field::new("count_", 1).unwrap_err().to_string(),
            "Invalid identifier: cannot start or end with an underscore"
        );
        assert_eq!(
            Field::new("a!@#", 1).unwrap_err().to_string(),
            "Invalid identifier: only letters, numbers and/or underscores are allowed"
        );
        assert_eq!(
            Field::new("count", 0).unwrap_err().to_string(),
            "Invalid argument: width cannot be zero"
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
        assert_eq!(fields.width(), 3);
        assert_eq!(Fields::try_from(vec![Field::new("a", 3)?])?, fields);

        let fields = Fields::new(vec![Field::new("a", 3)?, Field::new("b", 5)?])?;
        assert_eq!(fields.len(), 2);
        assert_eq!(fields.width(), 8);
        assert_eq!(fields.iter().len(), 2);
        assert_eq!(fields.fields(), &[Field::new("a", 3)?, Field::new("b", 5)?]);
        assert_eq!(
            fields.into_iter().map(|field| field.width()).sum::<usize>(),
            fields.width()
        );

        let fields = Fields::new(vec![Field::new("a", 3)?, Field::new("a", 3)?]);
        assert_eq!(
            fields.unwrap_err().to_string(),
            "Invalid argument: field identifiers must be case-insensitively unique within the set of named fields"
        );

        Ok(())
    }

    #[test]
    fn identifier() -> Result<(), Box<dyn error::Error>> {
        assert_eq!("asdf", Identifier::from_str("asdf")?.as_ref());
        assert_eq!(
            Identifier::try_from("asdf".to_string())?,
            Identifier::try_from("asdf")?
        );
        let identifier = Identifier::new("asdf")?;
        assert_eq!(identifier.to_string(), "asdf");
        assert_eq!(identifier, identifier);

        assert_eq!(
            Identifier::new("",).unwrap_err().to_string(),
            "Invalid identifier: cannot be empty"
        );
        assert_eq!(
            Identifier::new("1count").unwrap_err().to_string(),
            "Invalid identifier: cannot start with a digit"
        );
        assert_eq!(
            Identifier::new("_count").unwrap_err().to_string(),
            "Invalid identifier: cannot start or end with an underscore"
        );
        assert_eq!(
            Identifier::new("count_").unwrap_err().to_string(),
            "Invalid identifier: cannot start or end with an underscore"
        );
        assert_eq!(
            Identifier::new("a!@#").unwrap_err().to_string(),
            "Invalid identifier: only letters, numbers and/or underscores are allowed"
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
    fn physical_stream() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    #[test]
    fn signal_map() -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
}
