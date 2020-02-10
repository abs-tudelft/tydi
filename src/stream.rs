//! Stream-related traits, types and functions.

use crate::error::Error;
use std::{
    collections::HashSet, convert::TryFrom, error, fmt, iter::FromIterator, ops::Deref, slice::Iter,
};

/// A direction of a stream.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Indicating that the child stream carries data complementary to the data
    /// carried by the parent stream, in the same direction.
    Forward,
    /// Indicating that the child stream acts as a response channel for the
    /// parent stream.
    Reverse,
}

impl Reverse for Direction {
    /// Reverse this direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::stream::{Reverse, Reversed, Direction};
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

/// Logical stream interface complexity level.
///
/// This logical stream parameter specifies the guarantees a source makes about
/// how elements are transferred. Equivalently, it specifies the assumptions a
/// sink can safely make.
///
/// # Examples
///
/// ```rust
/// use tydi::stream::Complexity;
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
    /// use tydi::stream::Complexity;
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
    /// use tydi::stream::Complexity;
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
    /// use tydi::stream::Complexity;
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
    /// use tydi::stream::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.major(), 3);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    pub fn major(&self) -> usize {
        self.level[0]
    }
}

impl fmt::Display for Complexity {
    /// Display a complexity level as a version number. The levels are
    /// separated by periods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tydi::stream::Complexity;
    ///
    /// let c = Complexity::new(vec![3, 14])?;
    /// assert_eq!(c.to_string(), "3.14");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let mut level = self.level.iter().map(|x| x.to_string());
        if let Some(x) = level.next() {
            result.push_str(&x);
            level.for_each(|x| {
                result.push('.');
                result.push_str(&x);
            });
        }
        write!(f, "{}", result)
    }
}

/// Trait for something with a name.
pub trait Name {
    /// Returns the name.
    fn name(&self) -> &str;
}

/// Trait for something with a bit count.
pub trait BitCount {
    /// Returns the bit count.
    fn bit_count(&self) -> usize;
}

/// Type safe fields container builder.
pub(crate) struct FieldsBuilder<T>(Vec<T>);
impl<T> FieldsBuilder<T>
where
    T: Name,
{
    pub(crate) fn new() -> Self {
        FieldsBuilder(Vec::new())
    }

    pub(crate) fn add_field(&mut self, field: T) {
        self.0.push(field);
    }

    #[allow(dead_code)]
    pub(crate) fn with_field(mut self, field: T) -> Self {
        self.0.push(field);
        self
    }

    pub(crate) fn finish(self) -> Result<Fields<T>, Box<dyn error::Error>> {
        Fields::new(self.0)
    }
}

impl<T> Extend<T> for FieldsBuilder<T>
where
    T: Name,
{
    fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
        for elem in iter {
            self.add_field(elem)
        }
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
pub struct Fields<T>(Vec<T>);

impl<T> Default for Fields<T> {
    fn default() -> Self {
        Fields(vec![])
    }
}

impl<T> Fields<T>
where
    T: Name,
{
    /// Returns a new Fields container with provided Fields. Returns an error
    /// when the names of the provided fields are not case-insensitively
    /// unique.
    pub fn new(fields: impl IntoIterator<Item = T>) -> Result<Self, Box<dyn error::Error>> {
        let fields: Vec<T> = fields.into_iter().collect();
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
}

impl<T> Fields<T> {
    /// Returns the fields in this container.
    pub fn fields(&self) -> &[T] {
        self.0.as_ref()
    }

    /// Returns the number of fields in this container.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no fields in this container.
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

impl<T> BitCount for Fields<T>
where
    T: BitCount,
{
    fn bit_count(&self) -> usize {
        self.0.iter().map(T::bit_count).sum()
    }
}

impl<T> IntoIterator for Fields<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    /// Returns an iterator over the fields in this Fields container.
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Fields<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    /// Returns an iterator over the fields in this Fields container.
    fn into_iter(self) -> Self::IntoIter {
        self.fields().iter()
    }
}

impl<T> Extend<T> for Fields<T> {
    fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
        for item in iter {
            // TODO: key-check
            self.0.push(item);
        }
    }
}

impl<'a, T: 'a> Extend<&'a T> for Fields<T>
where
    T: Clone,
{
    fn extend<U: IntoIterator<Item = &'a T>>(&mut self, iter: U) {
        for item in iter {
            // TODO: key-check
            self.0.push(item.clone());
        }
    }
}

impl<T> Deref for Fields<T> {
    type Target = [T];

    /// Deref to a slice of the fields in this Fields container.
    fn deref(&self) -> &[T] {
        self.fields()
    }
}

impl<T> TryFrom<Vec<T>> for Fields<T>
where
    T: Name,
{
    type Error = Box<dyn error::Error>;

    /// Returns a new Fields container with provided Fields. Returns an error
    /// when the names of the provided fields are not case-insensitively
    /// unique.
    fn try_from(fields: Vec<T>) -> Result<Self, Self::Error> {
        Fields::new(fields)
    }
}

/// Trait for in-place reversing.
pub trait Reverse {
    fn reverse(&mut self);
}

/// Trait for construction of reversed values.
pub trait Reversed {
    fn reversed(&self) -> Self;
}

impl<T> Reversed for T
where
    T: Reverse + Clone,
{
    fn reversed(&self) -> T {
        let mut r = self.clone();
        r.reverse();
        r
    }
}

/// Function for validation of field names.
///
/// The following rules apply to field names, with double underscores being optional.
///
/// - The name of each field is a string consisting of letters, numbers, and/or underscores.
/// - The name cannot contain two or more consecutive underscores. [optional]
/// - The name cannot start or end with an underscore.
/// - The name cannot start with a digit.
/// - The name cannot be empty.
///
/// Returns an error when the provided name is an invalid field name.
/// Consecutive underscores are allowed when `double_underscores_allowed` is
/// set.
pub(crate) fn to_field_name(
    name: impl Into<String>,
    double_underscores_allowed: bool,
) -> Result<String, Box<dyn error::Error>> {
    let name = name.into();
    if name.is_empty() {
        Err(Box::new(Error::InvalidArgument(
            "name cannot be empty".to_string(),
        )))
    } else if name.chars().next().unwrap().is_ascii_digit() {
        Err(Box::new(Error::InvalidArgument(
            "name cannot start with a digit".to_string(),
        )))
    } else if name.starts_with('_') || name.ends_with('_') {
        Err(Box::new(Error::InvalidArgument(
            "name cannot start or end with an underscore".to_string(),
        )))
    } else if !double_underscores_allowed && name.contains("__") {
        Err(Box::new(Error::InvalidArgument(
            "name cannot contain two or more consecutive underscores".to_string(),
        )))
    } else if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c.eq(&'_'))
    {
        Err(Box::new(Error::InvalidArgument(
            "name must consist of letters, numbers, and/or underscores".to_string(),
        )))
    } else {
        Ok(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical::Field;
    use std::convert::TryInto;

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
    fn fields() -> Result<(), Box<dyn error::Error>> {
        let fields: Fields<Field> = Fields::new(vec![])?;
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
            fields.iter().map(|field| field.bit_count()).sum::<usize>(),
            fields.bit_count()
        );

        let fields = Fields::new(vec![Field::new("a", 3)?, Field::new("a", 3)?]);
        assert_eq!(
            fields.unwrap_err().to_string(),
            "Invalid argument: field names must be case-insensitively unique within the set of named fields"
        );

        Ok(())
    }
}
