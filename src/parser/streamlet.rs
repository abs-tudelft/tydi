use nom::{
    character::complete::newline,
    combinator::map,
    multi::{many1, separated_nonempty_list},
    sequence::tuple,
    IResult,
};

use crate::{parser::identifier, parser::logical::river_type, Streamlet};

/// Parses a Streamlet interface definition.
///
/// A streamlet interface definition consists of one or more input logical
/// stream types followed by one or more output logical stream types, separated
/// by a newline.
///
/// # Example
///
/// ```text
/// MuhStreamlet
///
/// Bits<1>
/// Bits<2>
///
/// Group<Bits<3>, Bits<4>>
/// Bits<4>
/// ```
///
pub fn streamlet_interface_definition(input: &str) -> IResult<&str, Streamlet> {
    map(
        tuple((
            identifier,
            many1(newline),
            separated_nonempty_list(newline, river_type),
            many1(newline),
            separated_nonempty_list(newline, river_type),
        )),
        |(identifier, _, input, _, output)| Streamlet {
            identifier,
            inputs: input,
            outputs: output,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::LogicalStream::{Bits, Group};

    use super::*;

    #[test]
    fn parse_streamlet_interface_definition() {
        assert_eq!(
            streamlet_interface_definition(
                r#"MuhStreamlet

a: Bits<1>
b: Bits<2>

c: Group<Bits<3>, Bits<4>>
d: Bits<4>"#
            )
            .unwrap(),
            (
                "",
                Streamlet {
                    identifier: "MuhStreamlet".to_string(),
                    inputs: vec![
                        Bits {
                            identifier: Some("a".to_string()),
                            width: 1,
                        },
                        Bits {
                            identifier: Some("b".to_string()),
                            width: 2,
                        }
                    ],
                    outputs: vec![
                        Group {
                            identifier: Some("c".to_string()),
                            inner: vec![
                                Bits {
                                    identifier: None,
                                    width: 3,
                                },
                                Bits {
                                    identifier: None,
                                    width: 4,
                                }
                            ],
                        },
                        Bits {
                            identifier: Some("d".to_string()),
                            width: 4,
                        }
                    ],
                }
            )
        );
    }
}
