#[cfg(feature = "parser")]
pub mod parser;

/// High level data types.
#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    /// Empty
    Empty,
    /// Prim<B>
    Prim(usize),
    /// Struct<T, U, ...>
    Struct(Vec<Data>),
    /// Tuple<T, n>
    Tuple(Box<Data>, usize),
    /// Seq<T>
    Seq(Box<Data>),
    /// Variant<T, U, ...>
    Variant(Vec<Data>),
}

/// River types.
#[derive(Clone, Debug, PartialEq)]
pub enum River {
    /// Bits<b>
    Bits(usize),
    /// Root<T, N, C, U>
    Root(Box<River>, RiverParameters),
    /// Group<T, U, ...>
    Group(Vec<River>),
    /// Dim<T, N, C, U>
    Dim(Box<River>, RiverParameters),
    /// New<T, N, C, U>
    New(Box<River>, RiverParameters),
    /// Rev<T, N, C, U>
    Rev(Box<River>, RiverParameters),
    /// Union<T, U, ...>
    Union(Vec<River>),
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
