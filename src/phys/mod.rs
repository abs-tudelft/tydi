//! Tydi physical streams.

/// Tydi stream interface complexity level.
// TODO(johanpel): implement Complexity "version numbering system" in parser
#[derive(Debug, Clone, PartialEq)]
pub struct Complexity {
    num: Vec<usize>
}

impl Complexity {
    // TODO(johanpel): See above todo, this needs fixing
    pub fn default() -> usize {
        0
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
    // only a width > 0 and no children.
}

impl BitField {
    /// Return the width of the sum of all bit fields in the bit field tree.
    pub fn width_recursive(&self) -> usize {
        self.width + self.children.iter().fold(0, |acc, x| acc + x.width_recursive())
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
            Dir::Upstream => Dir::Downstream
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
    pub name_parts: Vec<String>,
    /// Tree of bit fields contained within the elements of the physical stream.
    pub fields: BitField,
    /// The number of elements moved per transfer.
    pub elements_per_transfer: usize,
    /// The dimensionality, i.e. nesting level, of the elements.
    pub dimensionality: usize,
    /// Direction of the physical stream.
    pub dir: Dir,
    /// Complexity level of the physical stream.
    pub complexity: usize, // TODO(johanpel): this needs to be replaced with `Complexity` when the parser is fixed.
}
