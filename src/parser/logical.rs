use nom::{
    branch::alt,
    character::complete::char,
    combinator::{map, opt},
    sequence::{preceded, tuple},
    IResult,
};

use crate::{
    logical::{LogicalStream, LogicalStreamParameters},
    parser::{nonempty_comma_list, r#type, space_opt, usize},
    physical::Complexity,
};

macro_rules! river_type_parse_fn {
    ($ident:ident, $name:expr, $variant:path) => {
        pub fn $ident(input: &str) -> IResult<&str, LogicalStream> {
            river_type_parser($name, |(identifier, (river_type, river_parameters))| {
                $variant {
                    identifier,
                    inner: Box::new(river_type),
                    parameters: river_parameters.unwrap_or_default(),
                }
            })(input)
        }
    };
}

macro_rules! river_group_type_parse_fn {
    ($ident:ident, $name:expr, $variant:path) => {
        pub fn $ident(input: &str) -> IResult<&str, LogicalStream> {
            map(r#type($name, nonempty_comma_list(river_type)), |x| {
                $variant {
                    identifier: x.0,
                    inner: x.1,
                }
            })(input)
        }
    };
}

/// Returns a River type parser.
#[allow(clippy::needless_lifetimes)] // rust-lang/rust-clippy/issues/2944
fn river_type_parser<'a, F>(
    name: &'a str,
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, LogicalStream>
where
    F: Fn(
        (
            Option<String>,
            (LogicalStream, Option<LogicalStreamParameters>),
        ),
    ) -> LogicalStream,
{
    map(
        r#type(
            name,
            tuple((
                river_type,
                opt(preceded(space_opt(char(',')), river_parameters)),
            )),
        ),
        inner,
    )
}

/// Parses a LogicalStreamParameters.
pub fn river_parameters(input: &str) -> IResult<&str, LogicalStreamParameters> {
    map(
        tuple((
            usize,
            opt(space_opt(char(','))),
            opt(usize),
            opt(space_opt(char(','))),
            opt(usize),
        )),
        |(elements, _, complexity, _, userbits): (usize, _, Option<usize>, _, Option<usize>)| {
            LogicalStreamParameters {
                elements: Some(elements),
                complexity: match complexity {
                    None => None,
                    Some(num) => Some(Complexity::new_major(num)),
                },
                user_bits: userbits,
            }
        },
    )(input)
}

/// Parses a Bits<b>.
pub fn bits(input: &str) -> IResult<&str, LogicalStream> {
    map(r#type("Bits", usize), |(identifier, width)| {
        LogicalStream::Bits { identifier, width }
    })(input)
}

river_type_parse_fn!(root, "Root", LogicalStream::Root);
river_type_parse_fn!(dim, "Dim", LogicalStream::Dim);
river_type_parse_fn!(new, "New", LogicalStream::New);
river_type_parse_fn!(rev, "Rev", LogicalStream::Rev);
river_group_type_parse_fn!(group, "Group", LogicalStream::Group);
river_group_type_parse_fn!(r#union, "Union", LogicalStream::Union);

/// Parses a River type.
pub fn river_type(input: &str) -> IResult<&str, LogicalStream> {
    alt((r#union, rev, new, dim, group, root, bits))(input)
}

#[cfg(test)]
mod tests {
    use crate::logical::{LogicalStream, LogicalStreamParameters};

    use super::*;

    #[test]
    fn parse_river_parameters() {
        assert_eq!(
            river_parameters("3, 4, 5"),
            Ok((
                "",
                LogicalStreamParameters {
                    elements: Some(3),
                    complexity: Some(Complexity::new_major(4)),
                    user_bits: Some(5),
                }
            ))
        );
        assert!(river_parameters("").is_err());
        assert_eq!(
            river_parameters("1"),
            Ok((
                "",
                LogicalStreamParameters {
                    elements: Some(1),
                    complexity: None,
                    user_bits: None,
                }
            ))
        );
        assert_eq!(
            river_parameters("1,2"),
            Ok((
                "",
                LogicalStreamParameters {
                    elements: Some(1),
                    complexity: Some(Complexity::new_major(2)),
                    user_bits: None,
                }
            ))
        );
        assert_eq!(river_parameters("1,2"), river_parameters("1,2,"));
        assert_eq!(
            river_parameters("1,,3"),
            Ok((
                "",
                LogicalStreamParameters {
                    elements: Some(1),
                    complexity: None,
                    user_bits: Some(3),
                }
            ))
        );
    }

    #[test]
    fn parse_bits() {
        assert_eq!(
            bits("Bits<8>"),
            Ok((
                "",
                LogicalStream::Bits {
                    identifier: None,
                    width: 8,
                }
            ))
        );
        assert!(bits("Bits<>").is_err());
        assert!(bits("bits<8>").is_err());
    }

    #[test]
    fn parse_root() {
        assert_eq!(
            root("Root<Bits<8>, 1, 2, 3>"),
            Ok((
                "",
                LogicalStream::Root {
                    identifier: None,
                    inner: Box::new(LogicalStream::Bits {
                        identifier: None,
                        width: 8,
                    }),
                    parameters: LogicalStreamParameters {
                        elements: Some(1),
                        complexity: Some(Complexity::new_major(2)),
                        user_bits: Some(3),
                    },
                }
            ))
        );
        assert_eq!(
            root("Root<Bits<8>>"),
            Ok((
                "",
                LogicalStream::Root {
                    identifier: None,
                    inner: Box::new(LogicalStream::Bits {
                        identifier: None,
                        width: 8,
                    }),
                    parameters: LogicalStreamParameters::default(),
                }
            ))
        );
    }

    #[test]
    fn parse_group() {
        assert_eq!(
            group("Group<Bits<4>, Bits<8>>"),
            Ok((
                "",
                LogicalStream::Group {
                    identifier: None,
                    inner: vec![
                        LogicalStream::Bits {
                            identifier: None,
                            width: 4,
                        },
                        LogicalStream::Bits {
                            identifier: None,
                            width: 8,
                        }
                    ],
                }
            ))
        );
    }

    #[test]
    fn parse_dim() {
        assert_eq!(
            dim("Dim<Bits<8>, 1, 2, 3>"),
            Ok((
                "",
                LogicalStream::Dim {
                    identifier: None,
                    inner: Box::new(LogicalStream::Bits {
                        identifier: None,
                        width: 8,
                    }),
                    parameters: LogicalStreamParameters {
                        elements: Some(1),
                        complexity: Some(Complexity::new_major(2)),
                        user_bits: Some(3),
                    },
                }
            ))
        );
    }

    #[test]
    fn parse_new() {
        assert_eq!(
            new("New<Bits<7>, 3, 2, 1>"),
            Ok((
                "",
                LogicalStream::New {
                    identifier: None,

                    inner: Box::new(LogicalStream::Bits {
                        identifier: None,
                        width: 7,
                    }),
                    parameters: LogicalStreamParameters {
                        elements: Some(3),
                        complexity: Some(Complexity::new_major(2)),
                        user_bits: Some(1),
                    },
                }
            ))
        );
    }

    #[test]
    fn parse_rev() {
        assert_eq!(
            rev("Rev<Bits<8>, 11, 22, 33>"),
            Ok((
                "",
                LogicalStream::Rev {
                    identifier: None,

                    inner: Box::new(LogicalStream::Bits {
                        identifier: None,
                        width: 8,
                    }),
                    parameters: LogicalStreamParameters {
                        elements: Some(11),
                        complexity: Some(Complexity::new_major(22)),
                        user_bits: Some(33),
                    },
                }
            ))
        );
    }

    #[test]
    fn parse_union() {
        assert_eq!(
            union("Union<Bits<8>, Bits<4>>"),
            Ok((
                "",
                LogicalStream::Union {
                    identifier: None,
                    inner: vec![
                        LogicalStream::Bits {
                            identifier: None,
                            width: 8,
                        },
                        LogicalStream::Bits {
                            identifier: None,
                            width: 4,
                        }
                    ],
                }
            ))
        );
    }

    #[test]
    fn parse_river_type() {
        assert_eq!(
            river_type("Bits<8>"),
            Ok((
                "",
                LogicalStream::Bits {
                    identifier: None,
                    width: 8,
                }
            ))
        );
    }
}
