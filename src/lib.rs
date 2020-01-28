#[cfg(feature = "parser")]
pub mod parser;

/// High level data types.
#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    /// Empty
    Empty,
    /// Prim<B>
    Prim {
        identifier: Option<String>,
        width: usize,
    },
    /// Tuple<T, n>
    Tuple {
        identifier: Option<String>,
        child: Box<Data>,
        width: usize,
    },
    /// Seq<T>
    Seq {
        identifier: Option<String>,
        child: Box<Data>,
    },
    /// Struct<T, U, ...>
    Struct {
        identifier: Option<String>,
        children: Vec<Data>,
    },
    /// Variant<T, U, ...>
    Variant {
        identifier: Option<String>,
        children: Vec<Data>,
    },
}

/// River types.
#[derive(Clone, Debug, PartialEq)]
pub enum River {
    /// Bits<b>
    Bits {
        identifier: Option<String>,
        width: usize,
    },
    /// Root<T, N, C, U>
    Root {
        identifier: Option<String>,
        child: Box<River>,
        parameters: RiverParameters,
    },
    /// Dim<T, N, C, U>
    Dim {
        identifier: Option<String>,
        child: Box<River>,
        parameters: RiverParameters,
    },
    /// New<T, N, C, U>
    New {
        identifier: Option<String>,
        child: Box<River>,
        parameters: RiverParameters,
    },
    /// Rev<T, N, C, U>
    Rev {
        identifier: Option<String>,
        child: Box<River>,
        parameters: RiverParameters,
    },
    /// Group<T, U, ...>
    Group {
        identifier: Option<String>,
        children: Vec<River>,
    },
    /// Union<T, U, ...>
    Union {
        identifier: Option<String>,
        children: Vec<River>,
    },
}

impl River {
    /// Returns the combined width of the river types considering the
    /// RiverParameters for number of elements and userbits.
    pub fn width(&self) -> usize {
        match self {
            River::Bits { width, .. } => *width,
            River::Root {
                child, parameters, ..
            }
            | River::Dim {
                child, parameters, ..
            }
            | River::New {
                child, parameters, ..
            }
            | River::Rev {
                child, parameters, ..
            } => {
                parameters.elements.unwrap_or(1) * child.width() + parameters.userbits.unwrap_or(0)
            }
            River::Group { children, .. } => children.iter().map(|child| child.width()).sum(),
            River::Union { children, .. } => children
                .iter()
                .map(|child| child.width())
                .max()
                .unwrap_or(0),
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
            River::Root {
                identifier: None,
                child: Box::new(River::Bits {
                    identifier: None,
                    width: 3
                }),
                parameters: RiverParameters::default()
            }
            .width(),
            3
        );
        assert_eq!(
            River::Root {
                identifier: None,
                child: Box::new(River::Bits {
                    identifier: None,
                    width: 3
                }),
                parameters: RiverParameters {
                    elements: Some(2),
                    complexity: None,
                    userbits: None,
                }
            }
            .width(),
            6
        );
        assert_eq!(
            River::Root {
                identifier: None,
                child: Box::new(River::Bits {
                    identifier: None,
                    width: 3
                }),
                parameters: RiverParameters {
                    elements: Some(2),
                    complexity: None,
                    userbits: Some(3),
                }
            }
            .width(),
            9
        );
        assert_eq!(
            River::Group {
                identifier: None,
                children: vec![
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
                children: vec![
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
