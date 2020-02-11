//! Stream-related traits, types and functions.

use crate::error::Error;
use std::{cmp::Ordering, convert::TryFrom, error, fmt};

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
#[derive(Debug, Clone)]
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

impl PartialOrd for Complexity {
    fn partial_cmp(&self, other: &Complexity) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Complexity {
    /// A complexity number is higher than another when the leftmost integer is
    /// greater, and lower when the leftmost integer is lower. If the leftmost
    /// integer is equal, the next integer is checked recursively. If one
    /// complexity number has more entries than another, the shorter number is
    /// padded with zeros on the right.
    fn cmp(&self, other: &Complexity) -> Ordering {
        (0..self.level.len().max(other.level.len()))
            .map(|idx| {
                (
                    self.level.get(idx).unwrap_or(&0),
                    other.level.get(idx).unwrap_or(&0),
                )
            })
            .fold(None, |ord, (i, j)| match ord {
                Some(ord) => Some(ord),
                None => {
                    if i == j {
                        None
                    } else {
                        Some(i.cmp(j))
                    }
                }
            })
            .unwrap_or(Ordering::Equal)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn complexity() -> Result<(), Box<dyn error::Error>> {
        use std::convert::TryInto;

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
        assert!(!(c3 < c30));
        assert!(!(c3 > c30));
        assert_eq!(c3, c30);
        assert!(c31 < c311);
        assert!(c311 < c32);
        assert!(c32 < c4);
        assert_eq!(c4, c4);
        assert_eq!(c4, c400);
        assert_eq!(c400, c4);
        assert!(!(c400 > c4));
        assert!(!(c400 < c4));
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
}
