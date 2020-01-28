use crate::{
    parser::{nonempty_comma_list, r#type, space_opt, usize},
    River, RiverParameters,
};
use nom::{
    branch::alt,
    character::complete::char,
    combinator::{map, opt},
    sequence::{preceded, tuple},
    IResult,
};

macro_rules! river_type_parse_fn {
    ($ident:ident, $name:expr, $variant:path) => {
        pub fn $ident(input: &str) -> IResult<&str, River> {
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
        pub fn $ident(input: &str) -> IResult<&str, River> {
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
fn river_type_parser<'a, F>(name: &'a str, inner: F) -> impl Fn(&'a str) -> IResult<&'a str, River>
where
    F: Fn((Option<String>, (River, Option<RiverParameters>))) -> River,
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

/// Parses a RiverParameters.
pub fn river_parameters(input: &str) -> IResult<&str, RiverParameters> {
    map(
        tuple((
            usize,
            opt(space_opt(char(','))),
            opt(usize),
            opt(space_opt(char(','))),
            opt(usize),
        )),
        |(elements, _, complexity, _, userbits): (usize, _, Option<usize>, _, Option<usize>)| {
            RiverParameters {
                elements: Some(elements),
                complexity,
                userbits,
            }
        },
    )(input)
}

/// Parses a Bits<b>.
pub fn bits(input: &str) -> IResult<&str, River> {
    map(r#type("Bits", usize), |(identifier, width)| River::Bits {
        identifier,
        width,
    })(input)
}

river_type_parse_fn!(root, "Root", River::Root);
river_type_parse_fn!(dim, "Dim", River::Dim);
river_type_parse_fn!(new, "New", River::New);
river_type_parse_fn!(rev, "Rev", River::Rev);
river_group_type_parse_fn!(group, "Group", River::Group);
river_group_type_parse_fn!(r#union, "Union", River::Union);

/// Parses a River type.
pub fn river_type(input: &str) -> IResult<&str, River> {
    alt((r#union, rev, new, dim, group, root, bits))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RiverParameters;

    #[test]
    fn parse_river_parameters() {
        assert_eq!(
            river_parameters("3, 4, 5"),
            Ok((
                "",
                RiverParameters {
                    elements: Some(3),
                    complexity: Some(4),
                    userbits: Some(5)
                }
            ))
        );
        assert!(river_parameters("").is_err());
        assert_eq!(
            river_parameters("1"),
            Ok((
                "",
                RiverParameters {
                    elements: Some(1),
                    complexity: None,
                    userbits: None
                }
            ))
        );
        assert_eq!(
            river_parameters("1,2"),
            Ok((
                "",
                RiverParameters {
                    elements: Some(1),
                    complexity: Some(2),
                    userbits: None
                }
            ))
        );
        assert_eq!(river_parameters("1,2"), river_parameters("1,2,"));
        assert_eq!(
            river_parameters("1,,3"),
            Ok((
                "",
                RiverParameters {
                    elements: Some(1),
                    complexity: None,
                    userbits: Some(3)
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
                River::Bits {
                    identifier: None,
                    width: 8
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
                River::Root {
                    identifier: None,
                    inner: Box::new(River::Bits {
                        identifier: None,
                        width: 8
                    }),
                    parameters: RiverParameters {
                        elements: Some(1),
                        complexity: Some(2),
                        userbits: Some(3)
                    }
                }
            ))
        );
        assert_eq!(
            root("Root<Bits<8>>"),
            Ok((
                "",
                River::Root {
                    identifier: None,
                    inner: Box::new(River::Bits {
                        identifier: None,
                        width: 8
                    }),
                    parameters: RiverParameters::default()
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
                River::Group {
                    identifier: None,
                    inner: vec![
                        River::Bits {
                            identifier: None,
                            width: 4
                        },
                        River::Bits {
                            identifier: None,
                            width: 8
                        }
                    ]
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
                River::Dim {
                    identifier: None,
                    inner: Box::new(River::Bits {
                        identifier: None,
                        width: 8
                    }),
                    parameters: RiverParameters {
                        elements: Some(1),
                        complexity: Some(2),
                        userbits: Some(3)
                    }
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
                River::New {
                    identifier: None,

                    inner: Box::new(River::Bits {
                        identifier: None,
                        width: 7
                    }),
                    parameters: RiverParameters {
                        elements: Some(3),
                        complexity: Some(2),
                        userbits: Some(1)
                    }
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
                River::Rev {
                    identifier: None,

                    inner: Box::new(River::Bits {
                        identifier: None,
                        width: 8
                    }),
                    parameters: RiverParameters {
                        elements: Some(11),
                        complexity: Some(22),
                        userbits: Some(33)
                    }
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
                River::Union {
                    identifier: None,
                    inner: vec![
                        River::Bits {
                            identifier: None,
                            width: 8
                        },
                        River::Bits {
                            identifier: None,
                            width: 4
                        }
                    ]
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
                River::Bits {
                    identifier: None,
                    width: 8
                }
            ))
        );
    }
}
