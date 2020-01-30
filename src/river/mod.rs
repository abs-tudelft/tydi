//! Tydi streamspace types.

use crate::phys::{BitField, Complexity, Dir, Stream};

/// A potentially nested structure expressing a Streamspace type tree.
#[derive(Clone, Debug, PartialEq)]
pub enum River {
    /// Bits is a primitive element with `width` bits.
    Bits {
        identifier: Option<String>,
        width: usize,
    },
    /// Group concatenates all (nested) elements of inner `River` types into a
    /// single phys stream element.
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
    /// Rev creates a new phys stream with inner `River` types that flows
    /// in reverse direction w.r.t. its parent.
    Rev {
        identifier: Option<String>,
        inner: Box<River>,
        parameters: RiverParameters,
    },
    /// New creates a new phys stream of elements with inner `River` type
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

/// Apply elements-per-transfer and complexity from `params` to the first stream in a vector of
/// streams.
fn apply_params_to_first(streams: &mut Vec<Stream>, params: &RiverParameters) {
    if !streams.is_empty() {
        // First physical stream is the phys stream this Root is part of.
        streams[0].elements_per_transfer = params.elements.unwrap_or(1);
        streams[0].complexity = params.complexity.clone().unwrap_or_default();
        streams[0].user_bits = params.user_bits.unwrap_or(0);
    }
}

impl River {
    /// Return the identifier of the River.
    pub fn identifier(&self) -> Option<String> {
        match self {
            River::Bits { identifier, .. }
            | River::Group { identifier, .. }
            | River::Union { identifier, .. }
            | River::Dim { identifier, .. }
            | River::Rev { identifier, .. }
            | River::New { identifier, .. }
            | River::Root { identifier, .. } => identifier.clone(),
        }
    }

    /// Returns the combined width of the river types considering the
    /// RiverParameters for number of elements and user bits.
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

    /// Obtain sub-element bit fields resulting from the river type's immediate corresponding
    /// physical stream only. Ignores potentially nested physical streams.
    /// 'prefix' is used to prefix the bit fields.
    pub fn bit_fields(&self, prefix: Option<String>) -> Option<BitField> {
        match self {
            River::Group { identifier, inner } => {
                let suffix = identifier.clone().unwrap_or_else(|| "data".to_string());
                let id: String = match prefix {
                    None => suffix,
                    Some(pre) => format!("{}_{}", pre, suffix),
                };

                let mut result = BitField {
                    identifier: Some(id),
                    width: 0,
                    children: vec![],
                };
                // Iterate over all child river
                for child_river in inner.iter().enumerate() {
                    // Obtain child bitfields
                    let child_bitfields = child_river.1.bit_fields(None);
                    match child_bitfields {
                        None => (),
                        Some(child) => result.children.push(child),
                    }
                }
                Some(result)
            }
            River::Bits { identifier, width } => Some(BitField {
                identifier: identifier.clone(),
                width: *width,
                children: vec![], // no children
            }),
            _ => None,
        }
    }

    pub fn as_phys(&self, name: Option<String>) -> Vec<Stream> {
        // TODO(johanpel): propagate all parameters.
        match self {
            River::Root {
                identifier,
                inner,
                parameters,
            } => {
                // Return resulting streams from inner
                let mut result = inner.as_phys(identifier.clone());
                apply_params_to_first(&mut result, parameters);
                result
            }
            River::Dim {
                identifier,
                inner,
                parameters,
            } => {
                // Increase dimensionality of resulting streams
                let mut result = inner.as_phys(identifier.clone());
                for r in result.iter_mut() {
                    r.dimensionality += 1;
                }
                apply_params_to_first(&mut result, parameters);
                result
            }
            River::Rev {
                identifier,
                inner,
                parameters,
            } => {
                // Reverse child streams
                let mut result = inner.as_phys(identifier.clone());
                for r in result.iter_mut() {
                    r.dir.reverse()
                }
                apply_params_to_first(&mut result, parameters);
                result
            }
            River::New {
                identifier,
                inner,
                parameters,
            } => {
                // Return resulting streams from inner
                let mut result = inner.as_phys(identifier.clone());
                apply_params_to_first(&mut result, parameters);
                result
            }
            River::Bits { identifier, width } => {
                let new_stream = Stream {
                    identifier: name,
                    fields: BitField {
                        identifier: identifier.clone(),
                        width: *width,
                        children: vec![],
                    },
                    elements_per_transfer: 1,
                    dir: Dir::Downstream,
                    dimensionality: 0,
                    complexity: Complexity::default(),
                    user_bits: 0,
                };
                vec![new_stream]
            }
            River::Group { identifier, inner } => {
                let mut result = vec![];
                // Obtain all (nested) bit fields
                let bit_fields = self.bit_fields(identifier.clone());
                // If there are any bit fields, create a new stream
                if bit_fields.is_some() {
                    let new_stream = Stream {
                        identifier: name.clone(),
                        fields: bit_fields.unwrap_or_else(BitField::new_empty),
                        elements_per_transfer: 1,
                        dir: Dir::Downstream,
                        dimensionality: 0,
                        complexity: Complexity::default(),
                        user_bits: 0,
                    };
                    result.push(new_stream);
                }
                // Append the streams resulting from other fields.
                for field in inner.iter() {
                    match field {
                        // Skip bits type, since they will be added through bit_fields()
                        River::Bits { .. } => {}
                        // all other river types.
                        _ => result.extend(field.as_phys(name.clone()).into_iter()),
                    }
                }
                result
            }
            _ => unimplemented!(),
        }
    }
}

/// Parameters of River types.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RiverParameters {
    /// N: number of elements per handshake.
    pub elements: Option<usize>,
    /// C: complexity level.
    pub complexity: Option<Complexity>,
    /// U: number of user bits.
    pub user_bits: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn river_width() {
        assert_eq!(
            River::Bits {
                identifier: None,
                width: 3,
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
                        width: 7,
                    },
                    River::Bits {
                        identifier: None,
                        width: 16,
                    }
                ],
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
                        width: 3,
                    },
                    River::Bits {
                        identifier: None,
                        width: 4,
                    }
                ],
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
                        width: 3,
                    },
                    River::Bits {
                        identifier: None,
                        width: 4,
                    },
                    River::Dim {
                        identifier: None,
                        inner: Box::new(River::Bits {
                            identifier: None,
                            width: 10,
                        }),
                        parameters: Default::default(),
                    }
                ],
            }
            .width(),
            4
        );
    }

    #[test]
    fn test_river_bitfields() {
        // River of just bits.
        let r = River::Bits {
            identifier: Some("test".to_string()),
            width: 1,
        };
        let bf = r.bit_fields(None);
        assert!(bf.is_some());
        let bfu = bf.unwrap();
        assert_eq!(bfu.identifier, Some("test".to_string()));
        assert_eq!(bfu.width(), 1);
        assert_eq!(bfu.width_recursive(), 1);
    }

    #[test]
    fn test_river_bitfields_group() {
        let r = River::Group {
            identifier: Some("x".to_string()),
            inner: vec![
                River::Bits {
                    identifier: Some("a".to_string()),
                    width: 1,
                },
                River::Bits {
                    identifier: Some("b".to_string()),
                    width: 2,
                },
            ],
        };

        let bf = r.bit_fields(None);
        let bfu = &bf.unwrap();

        assert_eq!(bfu.children.len(), 2);
        assert_eq!(bfu.children[0].identifier, Some("a".to_string()));
        assert_eq!(bfu.children[0].width(), 1);
        assert_eq!(bfu.children[0].children.len(), 0);
        assert_eq!(bfu.children[1].identifier, Some("b".to_string()));
        assert_eq!(bfu.children[1].width(), 2);
        assert_eq!(bfu.children[1].children.len(), 0);
        assert_eq!(bfu.width_recursive(), 3);
    }

    #[test]
    fn test_river_bitfields_group_nested() {
        let r = River::Group {
            identifier: Some("x".to_string()),
            inner: vec![
                River::Bits {
                    identifier: Some("a".to_string()),
                    width: 1,
                },
                River::Group {
                    identifier: Some("b".to_string()),
                    inner: vec![
                        River::Bits {
                            identifier: Some("c".to_string()),
                            width: 2,
                        },
                        River::Bits {
                            identifier: Some("d".to_string()),
                            width: 3,
                        },
                    ],
                },
            ],
        };

        let bf = r.bit_fields(None);
        let bfu = bf.unwrap();

        assert_eq!(bfu.children.len(), 2);
        assert_eq!(bfu.children[0].identifier, Some("a".to_string()));
        assert_eq!(bfu.children[0].width(), 1);
        assert_eq!(bfu.children[0].children.len(), 0);
        assert_eq!(bfu.children[1].identifier, Some("b".to_string()));
        assert_eq!(bfu.children[1].width(), 0);
        assert_eq!(bfu.children[1].children.len(), 2);
        assert_eq!(
            bfu.children[1].children[0].identifier,
            Some("c".to_string())
        );
        assert_eq!(bfu.children[1].children[0].width(), 2);
        assert_eq!(bfu.children[1].children[0].children.len(), 0);
        assert_eq!(
            bfu.children[1].children[1].identifier,
            Some("d".to_string())
        );
        assert_eq!(bfu.children[1].children[1].width(), 3);
        assert_eq!(bfu.children[1].children[1].children.len(), 0);
        assert_eq!(bfu.width_recursive(), 6);
    }
}
