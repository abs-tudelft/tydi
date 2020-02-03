//! Tydi physical streams.

use crate::error::Error;
use std::{error, fmt};

/// Logical stream interface Complexity level.
///
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

impl Complexity {
    /// Constructs a new Complexity with provided level. The functions returns
    /// an error when the provided level iterator is empty.
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
                "complexity level can't be empty".to_string(),
            )))
        } else {
            Ok(Complexity {
                level: level.into_iter().collect(),
            })
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
        &self.level
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

impl Default for Complexity {
    fn default() -> Self {
        Complexity::new_major(0)
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

/// Element bit field.
#[derive(Debug, Clone)]
pub struct BitField {
    /// Identifier of this bit field.
    pub identifier: Option<String>,
    /// Number of bits in the bit field.
    pub width: usize,
    /// Potential child fields.
    pub children: Vec<BitField>,
    // TODO(johanpel): we need this tree to be either a bitfield with only children and width 0,
    // or only a width > 0 and no children. I.e. it was either a group or bits type as
    // streamspace type.
}

impl BitField {
    /// Return the width of the sum of all bit fields in the bit field tree.
    pub fn width_recursive(&self) -> usize {
        let v = self.width;
        let c = self
            .children
            .iter()
            .fold(0, |acc, x| acc + x.width_recursive());
        v + c
    }

    /// Return the width of only this bit field.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Return a new, empty bit field.
    pub fn new_empty() -> BitField {
        BitField {
            identifier: None,
            width: 0,
            children: vec![],
        }
    }

    /// Return a new, childless bit field with only a width.
    pub fn new(identifier: Option<String>, width: usize) -> BitField {
        BitField {
            identifier,
            width,
            children: vec![],
        }
    }
}

/// A direction of flow.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dir {
    /// From source to sink.
    Downstream,
    /// From sink to source.
    Upstream,
}

impl Dir {
    /// Obtain the reversed direction.
    pub fn reversed(self) -> Self {
        match self {
            Dir::Downstream => Dir::Upstream,
            Dir::Upstream => Dir::Downstream,
        }
    }
    /// In-place reverse.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
}

/// A Tydi physical stream.
#[derive(Debug)]
pub struct PhysicalStream {
    /// Name of the physical stream. Stored as a vector of strings to allow various types of
    /// joins for different back-ends and preferences.
    pub identifier: Vec<String>,
    /// Tree of bit fields contained within the elements of the physical stream.
    pub fields: BitField,
    /// The number of elements moved per transfer.
    pub elements_per_transfer: usize,
    /// The dimensionality, i.e. nesting level, of the elements.
    pub dimensionality: usize,
    /// The user bits.
    pub user_bits: usize,
    /// Direction of the physical stream.
    pub dir: Dir,
    /// Complexity level of the physical stream.
    pub complexity: Complexity,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    #[test]
    fn complexity() -> Result<(), Box<dyn error::Error>> {
        let empty = Complexity::new(vec![]);
        assert_eq!(
            empty.unwrap_err().to_string(),
            "Invalid argument: complexity level can\'t be empty"
        );

        let c = Complexity::default();
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
    fn test_bitfield_recursive_width() {
        let bf = BitField {
            identifier: None,
            width: 0,
            children: vec![
                BitField {
                    identifier: None,
                    width: 1,
                    children: vec![],
                },
                BitField {
                    identifier: None,
                    width: 0,
                    children: vec![
                        BitField {
                            identifier: None,
                            width: 2,
                            children: vec![],
                        },
                        BitField {
                            identifier: None,
                            width: 3,
                            children: vec![],
                        },
                    ],
                },
            ],
        };
        assert_eq!(bf.width_recursive(), 6);
    }
}
