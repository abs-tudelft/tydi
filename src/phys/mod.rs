//! Tydi physical streams.

/// Tydi stream interface complexity level.
#[derive(Debug, Clone, PartialEq)]
pub struct Complexity {
    num: Vec<usize>,
}

impl Complexity {
    pub fn new_major(num: usize) -> Self {
        Complexity { num: vec![num] }
    }
}

impl Default for Complexity {
    fn default() -> Self {
        Complexity { num: vec![0] }
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
    // TODO(johanpel): we need this tree to be either a bitfield with only children and width 0, or
    // only a width > 0 and no children. I.e. it was either a group or bits type.
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
pub struct Stream {
    /// Name of the physical stream.
    pub identifier: Option<String>,
    /// Tree of bit fields contained within the elements of the physical stream.
    pub fields: BitField,
    /// The number of elements moved per transfer.
    pub elements_per_transfer: usize,
    /// The dimensionality, i.e. nesting level, of the elements.
    pub dimensionality: usize,
    /// Direction of the physical stream.
    pub dir: Dir,
    /// Complexity level of the physical stream.
    pub complexity: Complexity,
}

#[cfg(test)]
mod test {
    use crate::phys::*;

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
