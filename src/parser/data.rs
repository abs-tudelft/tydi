use crate::{
    parser::{nonempty_comma_list, r#type, space_opt, usize},
    Data,
};
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::map,
    sequence::separated_pair, IResult,
};

/// Parses an Empty.
pub fn empty(input: &str) -> IResult<&str, Data> {
    map(tag("Empty"), |_| Data::Empty)(input)
}

/// Parses a Prim<B>.
pub fn prim(input: &str) -> IResult<&str, Data> {
    map(r#type("Prim", usize), |(identifier, width)| Data::Prim {
        identifier,
        width,
    })(input)
}

/// Parses a Struct<T, U, ...>.
pub fn r#struct(input: &str) -> IResult<&str, Data> {
    map(
        r#type("Struct", nonempty_comma_list(data_type)),
        |(identifier, children)| Data::Struct {
            identifier,
            children,
        },
    )(input)
}

/// Parses a Tuple<T, n>.
pub fn tuple(input: &str) -> IResult<&str, Data> {
    map(
        r#type(
            "Tuple",
            separated_pair(data_type, space_opt(char(',')), usize),
        ),
        |(identifier, (data_type, width))| Data::Tuple {
            identifier,
            child: Box::new(data_type),
            width,
        },
    )(input)
}

/// Parses a Seq<T>.
pub fn seq(input: &str) -> IResult<&str, Data> {
    map(r#type("Seq", data_type), |(identifier, data_type)| {
        Data::Seq {
            identifier,
            child: Box::new(data_type),
        }
    })(input)
}

/// Parses a Variant<T, U, ...>.
pub fn variant(input: &str) -> IResult<&str, Data> {
    map(
        r#type("Variant", nonempty_comma_list(data_type)),
        |(identifier, children)| Data::Variant {
            identifier,
            children,
        },
    )(input)
}

/// Parses a Data type.
pub fn data_type(input: &str) -> IResult<&str, Data> {
    alt((variant, seq, tuple, r#struct, prim, empty))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert_eq!(empty("Empty"), Ok(("", Data::Empty)));
    }

    #[test]
    fn parse_prim() {
        assert_eq!(
            prim("Prim<8>"),
            Ok((
                "",
                Data::Prim {
                    identifier: None,
                    width: 8
                }
            ))
        );
        assert!(prim("Prim<>").is_err());
        assert!(prim("prim<8>").is_err());
        assert_eq!(
            prim("a: Prim<3>"),
            Ok((
                "",
                Data::Prim {
                    identifier: Some("a".to_string()),
                    width: 3
                }
            ))
        );
    }

    #[test]
    fn parse_struct() {
        assert_eq!(
            r#struct("Struct<Prim<3>>"),
            Ok((
                "",
                Data::Struct {
                    identifier: None,
                    children: vec![Data::Prim {
                        identifier: None,
                        width: 3
                    }]
                }
            ))
        );
        assert_eq!(
            r#struct("Struct<Struct<Prim<3>>>"),
            Ok((
                "",
                Data::Struct {
                    identifier: None,
                    children: vec![Data::Struct {
                        identifier: None,
                        children: vec![Data::Prim {
                            identifier: None,
                            width: 3
                        }]
                    }]
                }
            ))
        );
        assert_eq!(
            r#struct("Struct<Struct<Prim<3>,Prim<8>>>"),
            Ok((
                "",
                Data::Struct {
                    identifier: None,
                    children: vec![Data::Struct {
                        identifier: None,
                        children: vec![
                            Data::Prim {
                                identifier: None,
                                width: 3
                            },
                            Data::Prim {
                                identifier: None,
                                width: 8
                            }
                        ]
                    }]
                }
            ))
        );
        assert_eq!(
            r#struct("Struct<Struct<Prim<3>,Prim<8>>>"),
            r#struct("Struct<Struct<Prim<3>, Prim<8>>>"),
        );
        assert_eq!(
            r#struct("a: Struct<b:Prim<3>>"),
            Ok((
                "",
                Data::Struct {
                    identifier: Some("a".to_string()),
                    children: vec![Data::Prim {
                        identifier: Some("b".to_string()),
                        width: 3
                    }]
                }
            ))
        );
        assert_eq!(
            r#struct("a: Struct<b:Prim<3>>"),
            r#struct("a:Struct<b:   Prim<3>>")
        );
    }

    #[test]
    fn parse_tuple() {
        assert_eq!(
            tuple("Tuple<Prim<8>,4>"),
            Ok((
                "",
                Data::Tuple {
                    identifier: None,
                    child: Box::new(Data::Prim {
                        identifier: None,
                        width: 8
                    }),
                    width: 4
                }
            ))
        );
        assert_eq!(
            tuple("c: Tuple<c: Prim<8>,4>"),
            Ok((
                "",
                Data::Tuple {
                    identifier: Some("c".to_string()),
                    child: Box::new(Data::Prim {
                        identifier: Some("c".to_string()),
                        width: 8
                    }),
                    width: 4
                }
            ))
        );
    }

    #[test]
    fn parse_seq() {
        assert_eq!(
            seq("Seq<Tuple<a: Prim<8>,4>>"),
            Ok((
                "",
                Data::Seq {
                    identifier: None,
                    child: Box::new(Data::Tuple {
                        identifier: None,
                        child: Box::new(Data::Prim {
                            identifier: Some("a".to_string()),
                            width: 8
                        }),
                        width: 4
                    })
                }
            ))
        );
    }

    #[test]
    fn parse_variant() {
        assert_eq!(
            variant("Variant<Prim<8>, Seq<Tuple<byte: Prim<8>,4>>>"),
            Ok((
                "",
                Data::Variant {
                    identifier: None,
                    children: vec![
                        Data::Prim {
                            identifier: None,
                            width: 8
                        },
                        Data::Seq {
                            identifier: None,
                            child: Box::new(Data::Tuple {
                                identifier: None,
                                child: Box::new(Data::Prim {
                                    identifier: Some("byte".to_string()),
                                    width: 8
                                }),
                                width: 4
                            })
                        }
                    ]
                }
            ))
        );
    }

    #[test]
    fn parse_data_type() {
        assert_eq!(
            data_type("Prim<8>"),
            Ok((
                "",
                Data::Prim {
                    identifier: None,
                    width: 8
                }
            ))
        );
        assert_eq!(
            data_type("Struct<Prim<4>, Prim<4>>"),
            Ok((
                "",
                Data::Struct {
                    identifier: None,
                    children: vec![
                        Data::Prim {
                            identifier: None,
                            width: 4
                        },
                        Data::Prim {
                            identifier: None,
                            width: 4
                        }
                    ]
                }
            ))
        );
        assert_eq!(
            data_type("Tuple<Prim<8>, 4>"),
            Ok((
                "",
                Data::Tuple {
                    identifier: None,
                    child: Box::new(Data::Prim {
                        identifier: None,
                        width: 8
                    }),
                    width: 4
                }
            ))
        );
        assert_eq!(
            data_type("Seq<Prim<4>>"),
            Ok((
                "",
                Data::Seq {
                    identifier: None,
                    child: Box::new(Data::Prim {
                        identifier: None,
                        width: 4
                    })
                }
            ))
        );
        assert_eq!(
            data_type("Variant<Prim<4>,Prim<4>>"),
            Ok((
                "",
                Data::Variant {
                    identifier: None,
                    children: vec![
                        Data::Prim {
                            identifier: None,
                            width: 4
                        },
                        Data::Prim {
                            identifier: None,
                            width: 4
                        }
                    ]
                }
            ))
        );
    }
}
