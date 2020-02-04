//! Logical streams.
//!
//! [Reference]
//!
//! [Reference]: https://abs-tudelft.github.io/tydi/specification/logical.html

use crate::physical::{Complexity, Field, PhysicalStream};
use std::fmt;

/// A potentially nested structure expressing a logical stream type tree.
#[derive(Clone, Debug, PartialEq)]
pub enum LogicalStream {
    /// Bits is a primitive element with `width` bits.
    Bits {
        identifier: Option<String>,
        width: usize,
    },
    /// Group concatenates all (nested) elements of inner `LogicalStream` types into a
    /// single phys stream element.
    Group {
        identifier: Option<String>,
        inner: Vec<LogicalStream>,
    },
    /// Union defines a `B`-bits element, where `B` is the maximum `width`
    /// value of the `inner` LogicalStream types.
    Union {
        identifier: Option<String>,
        inner: Vec<LogicalStream>,
    },
    /// Dim creates a streamspace of elements with inner `LogicalStream` type in the
    /// next dimension w.r.t. its parent.
    Dim {
        identifier: Option<String>,
        inner: Box<LogicalStream>,
        parameters: LogicalStreamParameters,
    },
    /// Rev creates a new phys stream with inner `LogicalStream` types that flows
    /// in reverse direction w.r.t. its parent.
    Rev {
        identifier: Option<String>,
        inner: Box<LogicalStream>,
        parameters: LogicalStreamParameters,
    },
    /// New creates a new phys stream of elements with inner `LogicalStream` type
    /// in the parent space `D_{p}`.
    New {
        identifier: Option<String>,
        inner: Box<LogicalStream>,
        parameters: LogicalStreamParameters,
    },
    /// Root creates an initial streamspace `D_{0}`.
    Root {
        identifier: Option<String>,
        inner: Box<LogicalStream>,
        parameters: LogicalStreamParameters,
    },
}

/// Apply elements-per-transfer and complexity from `params` to the first stream in a vector of
/// streams.
fn apply_params_to_first(streams: &mut Vec<PhysicalStream>, params: &LogicalStreamParameters) {
    if !streams.is_empty() {
        // First physical stream is the phys stream this Root is part of.
        streams[0].elements_per_transfer = params.elements.unwrap_or(1);
        streams[0].complexity = params.complexity.clone().unwrap_or_default();
        streams[0].user_bits = params.user_bits.unwrap_or(0);
    }
}

fn push_some<T: Clone>(v: &[T], e: &Option<T>) -> Vec<T> {
    let mut result = v.to_owned();
    match e {
        None => (),
        Some(s) => result.push(s.clone()),
    }
    result
}

impl LogicalStream {
    /// Return the identifier of the LogicalStream.
    pub fn identifier(&self) -> Option<String> {
        match self {
            LogicalStream::Bits { identifier, .. }
            | LogicalStream::Group { identifier, .. }
            | LogicalStream::Union { identifier, .. }
            | LogicalStream::Dim { identifier, .. }
            | LogicalStream::Rev { identifier, .. }
            | LogicalStream::New { identifier, .. }
            | LogicalStream::Root { identifier, .. } => identifier.clone(),
        }
    }

    /// Returns the combined width of the LogicalStream types considering the
    /// LogicalStreamParameters for number of elements and user bits.
    pub fn width(&self) -> usize {
        match self {
            LogicalStream::Bits { width, .. } => *width,
            LogicalStream::Group { inner, .. } => inner.iter().map(|inner| inner.width()).sum(),
            LogicalStream::Union { inner, .. } => {
                inner.iter().map(|inner| inner.width()).max().unwrap_or(0)
            }
            LogicalStream::Dim { .. }
            | LogicalStream::Rev { .. }
            | LogicalStream::New { .. }
            | LogicalStream::Root { .. } => 0,
        }
    }

    /// Obtain sub-element bit fields resulting from the LogicalStream type's immediate corresponding
    /// physical stream only. Ignores potentially nested physical streams.
    /// 'prefix' is used to prefix the bit fields.
    pub fn bit_fields(&self, prefix: Option<String>) -> Option<BitField> {
        match self {
            LogicalStream::Group { identifier, inner } => {
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
                // Iterate over all child LogicalStream
                for child_logical_stream in inner.iter().enumerate() {
                    // Obtain child bitfields
                    let child_bitfields = child_logical_stream.1.bit_fields(None);
                    match child_bitfields {
                        None => (),
                        Some(child) => result.children.push(child),
                    }
                }
                Some(result)
            }
            LogicalStream::Bits { identifier, width } => Some(BitField {
                identifier: identifier.clone(),
                width: *width,
                children: vec![], // no children
            }),
            _ => None,
        }
    }

    /// Convert this LogicalStream to Physical Streams.
    ///
    /// This can potentially generate multiple physical streams.
    pub fn as_phys(&self, name: Vec<String>) -> Vec<PhysicalStream> {
        // TODO(johanpel):  this flattens the LogicalStream type structure but we could consider allowing
        //                  physical streams to be nested.
        match self {
            LogicalStream::Root {
                identifier,
                inner,
                parameters,
            } => {
                // Return resulting streams from inner
                let mut result = inner.as_phys(push_some(&name, identifier));

                apply_params_to_first(&mut result, parameters);
                result
            }
            LogicalStream::Dim {
                identifier,
                inner,
                parameters,
            } => {
                // Increase dimensionality of resulting streams
                let mut result = inner.as_phys(push_some(&name, identifier));
                for r in result.iter_mut() {
                    r.dimensionality += 1;
                }
                apply_params_to_first(&mut result, parameters);
                result
            }
            LogicalStream::Rev {
                identifier,
                inner,
                parameters,
            } => {
                // Reverse child streams
                let mut result = inner.as_phys(push_some(&name, identifier));
                for r in result.iter_mut() {
                    r.dir.reverse()
                }
                apply_params_to_first(&mut result, parameters);
                result
            }
            LogicalStream::New {
                identifier,
                inner,
                parameters,
            } => {
                // Return resulting streams from inner
                let mut result = inner.as_phys(push_some(&name, identifier));
                apply_params_to_first(&mut result, parameters);
                result
            }
            LogicalStream::Bits { identifier, width } => {
                let new_stream = PhysicalStream {
                    identifier: push_some(&name, identifier),
                    fields: BitField {
                        identifier: None,
                        width: *width,
                        children: vec![],
                    },
                    elements_per_transfer: 1,
                    dir: Direction::Downstream,
                    dimensionality: 0,
                    complexity: Complexity::new_major(0),
                    user_bits: 0,
                };
                vec![new_stream]
            }
            LogicalStream::Group { identifier, inner } => {
                let mut result = vec![];
                // Obtain all (nested) bit fields
                let bit_fields = self.bit_fields(identifier.clone());
                // If there are any bit fields, create a new stream
                if bit_fields.is_some() {
                    let new_stream = PhysicalStream {
                        identifier: push_some(&name, identifier),
                        fields: bit_fields.unwrap_or_else(BitField::new_empty),
                        elements_per_transfer: 1,
                        dir: Direction::Downstream,
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
                        LogicalStream::Bits { .. } => {}
                        // all other LogicalStream types.
                        _ => result.extend(field.as_phys(name.clone()).into_iter()),
                    }
                }
                result
            }
            _ => unimplemented!(),
        }
    }
}

/// Parameters of LogicalStream types.
#[derive(Clone, Default, PartialEq)]
pub struct LogicalStreamParameters {
    /// N: number of elements per handshake.
    pub elements: Option<usize>,
    /// C: complexity level.
    pub complexity: Option<Complexity>,
    /// U: number of user bits.
    pub user_bits: Option<usize>,
}

impl fmt::Debug for LogicalStreamParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(E={},C={},U={})",
            self.elements.unwrap_or(1),
            self.complexity.clone().unwrap_or_default(),
            self.user_bits.unwrap_or(0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_stream_width() {
        assert_eq!(
            LogicalStream::Bits {
                identifier: None,
                width: 3,
            }
            .width(),
            3
        );
        assert_eq!(
            LogicalStream::Group {
                identifier: None,
                inner: vec![
                    LogicalStream::Bits {
                        identifier: None,
                        width: 7,
                    },
                    LogicalStream::Bits {
                        identifier: None,
                        width: 16,
                    }
                ],
            }
            .width(),
            23
        );
        assert_eq!(
            LogicalStream::Group {
                identifier: None,
                inner: vec![
                    LogicalStream::Bits {
                        identifier: None,
                        width: 3,
                    },
                    LogicalStream::Bits {
                        identifier: None,
                        width: 4,
                    }
                ],
            }
            .width(),
            7
        );
        assert_eq!(
            LogicalStream::Union {
                identifier: None,
                inner: vec![
                    LogicalStream::Bits {
                        identifier: None,
                        width: 3,
                    },
                    LogicalStream::Bits {
                        identifier: None,
                        width: 4,
                    },
                    LogicalStream::Dim {
                        identifier: None,
                        inner: Box::new(LogicalStream::Bits {
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
    fn test_logical_stream_bitfields() {
        // LogicalStream of just bits.
        let r = LogicalStream::Bits {
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
    fn test_logical_stream_bitfields_group() {
        let r = LogicalStream::Group {
            identifier: Some("x".to_string()),
            inner: vec![
                LogicalStream::Bits {
                    identifier: Some("a".to_string()),
                    width: 1,
                },
                LogicalStream::Bits {
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
    fn test_logical_stream_bitfields_group_nested() {
        let r = LogicalStream::Group {
            identifier: Some("x".to_string()),
            inner: vec![
                LogicalStream::Bits {
                    identifier: Some("a".to_string()),
                    width: 1,
                },
                LogicalStream::Group {
                    identifier: Some("b".to_string()),
                    inner: vec![
                        LogicalStream::Bits {
                            identifier: Some("c".to_string()),
                            width: 2,
                        },
                        LogicalStream::Bits {
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
