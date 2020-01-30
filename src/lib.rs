use crate::river::River;

pub mod generator;
#[cfg(feature = "parser")]
pub mod parser;
pub mod phys;
pub mod river;

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

/// Streamlet interface definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Streamlet {
    pub identifier: String,
    pub inputs: Vec<River>,
    pub outputs: Vec<River>,
}
