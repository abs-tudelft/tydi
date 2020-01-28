#[cfg(feature = "parser")]
pub mod parser;

/// High-level data types.
#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    /// No value, empty set.
    Empty,
    /// Primitive element containing `width` bits of information.
    Prim {
        identifier: Option<String>,
        width: usize,
    },
    /// A fixed-length aggregate type. An instance is a sequence with `width`
    /// instances of the inner `Data` type.
    Tuple {
        identifier: Option<String>,
        inner: Box<Data>,
        width: usize,
    },
    /// A variable-length aggregate type. An instance is a sequence with a
    /// variable number of instances of the inner `Data` type.
    Seq {
        identifier: Option<String>,
        inner: Box<Data>,
    },
    /// A composite type. An instance is a set with one instance for all inner
    /// `Data` types.
    Struct {
        identifier: Option<String>,
        inner: Vec<Data>,
    },
    /// A variant type. An instance is one of the inner `Data` types with a tag
    /// indicating the variant.
    Variant {
        identifier: Option<String>,
        inner: Vec<Data>,
    },
    // TODO: add map type
}

/// Streamspace types.
#[derive(Clone, Debug, PartialEq)]
pub enum River {
    /// Bits is a primitive element with `width` bits.
    Bits {
        identifier: Option<String>,
        width: usize,
    },
    /// Group concatenates all (nested) elements of inner `River` types into a
    /// single physical stream element.
    Group {
        identifier: Option<String>,
        inner: Vec<River>,
    },
    /// Union defines a `B`-bits element, where `B` is the maximum `width`
    /// value of the `inner` River types.
    Union {
        identifier: Option<String>,
        inner: Vec<River>,
    },
    /// Dim creates a streamspace of elements with inner `River` type in the
    /// next dimension w.r.t. its parent.
    Dim {
        identifier: Option<String>,
        inner: Box<River>,
        parameters: RiverParameters,
    },
    /// Rev creates a new physical stream with inner `River` types that flows
    /// in reverse direction w.r.t. its parent.
    Rev {
        identifier: Option<String>,
        inner: Box<River>,
        parameters: RiverParameters,
    },
    /// New creates a new physical stream of elements with inner `River` type
    /// in the parent space `D_{p}`.
    New {
        identifier: Option<String>,
        inner: Box<River>,
        parameters: RiverParameters,
    },
    /// Root creates an initial streamspace `D_{0}`.
    Root {
        identifier: Option<String>,
        inner: Box<River>,
        parameters: RiverParameters,
    },
}

impl River {
    /// Returns the combined width of the river types considering the
    /// RiverParameters for number of elements and userbits.
    pub fn width(&self) -> usize {
        match self {
            River::Bits { width, .. } => *width,
            River::Group { inner, .. } => inner.iter().map(|inner| inner.width()).sum(),
            River::Union { inner, .. } => {
                inner.iter().map(|inner| inner.width()).max().unwrap_or(0)
            }
            River::Dim { .. } | River::Rev { .. } | River::New { .. } | River::Root { .. } => 0,
        }
    }
}

/// Parameters of River types.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RiverParameters {
    /// N: number of elements per handshake.
    pub elements: Option<usize>,
    /// C: complexity level.
    pub complexity: Option<usize>,
    /// U: number of user bits.
    pub userbits: Option<usize>,
}

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    pub input: Vec<River>,
    pub output: Vec<River>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn river_width() {
        assert_eq!(
            River::Bits {
                identifier: None,
                width: 3
            }
            .width(),
            3
        );
        assert_eq!(
            River::Group {
                identifier: None,
                inner: vec![
                    River::Bits {
                        identifier: None,
                        width: 7
                    },
                    River::Bits {
                        identifier: None,
                        width: 16
                    }
                ]
            }
            .width(),
            23
        );
        assert_eq!(
            River::Group {
                identifier: None,
                inner: vec![
                    River::Bits {
                        identifier: None,
                        width: 3
                    },
                    River::Bits {
                        identifier: None,
                        width: 4
                    }
                ]
            }
            .width(),
            7
        );
        assert_eq!(
            River::Union {
                identifier: None,
                inner: vec![
                    River::Bits {
                        identifier: None,
                        width: 3
                    },
                    River::Bits {
                        identifier: None,
                        width: 4
                    }
                ]
            }
            .width(),
            4
        );
    }
}
