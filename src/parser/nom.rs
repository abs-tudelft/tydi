//! Nom-based parsers for Streamlet Definition Files.

use crate::design::{Interface, Mode, Streamlet};
use crate::logical::{Direction, Group, LogicalStreamType, Stream, Synchronicity, Union};
use crate::physical::Complexity;
use crate::{Name, PositiveReal};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{digit1, multispace1, one_of},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, many1, separated_list},
    number::complete::float,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};
use std::collections::HashMap;

// #[derive(Debug, PartialEq)]
// pub struct ParserError<I> {
//     kind: ErrorKind<I>,
//     backtrace: Vec<ErrorKind<I>>,
// }
//
// impl<I> ParserError<I> {
//     fn backtrace(&mut self, other: ParserError<I>) {
//         self.backtrace.push(other.kind);
//     }
// }
//
// #[derive(Debug, PartialEq)]
// enum ErrorKind<I> {
//     Nom(I, nom::error::ErrorKind),
//     BadName(String),
// }
//
// impl<I> ParseError<I> for ParserError<I> {
//     fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
//         ParserError {
//             kind: ErrorKind::Nom(input, kind),
//             backtrace: vec![],
//         }
//     }
//
//     fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
//         other.backtrace(Self::from_error_kind(input, kind));
//         other
//     }
// }

type Result<I, T> = nom::IResult<I, T, nom::error::VerboseError<I>>;

fn ws0(input: &str) -> Result<&str, Vec<&str>> {
    many0(alt((multispace1, comment)))(input)
}

fn ws1(input: &str) -> Result<&str, Vec<&str>> {
    many1(alt((multispace1, comment)))(input)
}

fn w<'a, T>(f: impl Fn(&'a str) -> Result<&'a str, T>) -> impl Fn(&'a str) -> Result<&'a str, T> {
    terminated(f, ws0)
}

pub fn name(input: &str) -> Result<&str, Name> {
    map_res(
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
        |name: &str| Name::try_new(name).map_err(|_| ()),
    )(input)
}

pub fn comment(input: &str) -> Result<&str, &str> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}

pub fn bool(input: &str) -> Result<&str, bool> {
    map(alt((tag("true"), tag("false"))), |x: &str| {
        x.parse::<bool>().unwrap()
    })(input)
}

pub fn null(input: &str) -> Result<&str, LogicalStreamType> {
    map(tag("Null"), |_| LogicalStreamType::Null)(input)
}

pub fn bits(input: &str) -> Result<&str, LogicalStreamType> {
    map_res(
        delimited(w(tag("Bits<")), w(digit1), tag(">")),
        |x: &str| LogicalStreamType::try_new_bits(x.parse().unwrap()).map_err(|_| ()),
    )(input)
}

pub fn logical_stream_type(input: &str) -> Result<&str, LogicalStreamType> {
    alt((null, bits, group, union, stream))(input)
}

fn fields(input: &str) -> Result<&str, Vec<(Name, LogicalStreamType)>> {
    separated_list(
        w(tag(",")),
        separated_pair(w(name), w(tag(":")), w(logical_stream_type)),
    )(input)
}

pub fn group(input: &str) -> Result<&str, LogicalStreamType> {
    map_res(
        delimited(w(tag("Group<")), w(fields), tag(">")),
        |fields: Vec<(Name, LogicalStreamType)>| {
            Group::try_new(fields).map(Into::into).map_err(|_| ())
        },
    )(input)
}

pub fn union(input: &str) -> Result<&str, LogicalStreamType> {
    map_res(
        delimited(w(tag("Union<")), w(fields), tag(">")),
        |fields: Vec<(Name, LogicalStreamType)>| {
            Union::try_new(fields).map(Into::into).map_err(|_| ())
        },
    )(input)
}

pub fn complexity(input: &str) -> Result<&str, Complexity> {
    map_res(separated_list(w(tag(".")), digit1), |level: Vec<&str>| {
        Complexity::new(level.iter().map(|x| x.parse().unwrap())).map_err(|_| ())
    })(input)
}

pub fn synchronicity(input: &str) -> Result<&str, Synchronicity> {
    map(
        alt((
            tag("Sync"),
            tag("Flatten"),
            tag("Desync"),
            tag("FlatDesync"),
        )),
        |x: &str| x.parse().unwrap(),
    )(input)
}

pub fn direction(input: &str) -> Result<&str, Direction> {
    map(alt((tag("Forward"), tag("Reverse"))), |x: &str| {
        x.parse().unwrap()
    })(input)
}

pub fn stream(input: &str) -> Result<&str, LogicalStreamType> {
    map_res(
        tuple((
            w(tag("Stream<")),
            w(logical_stream_type),
            opt(preceded(
                w(tag(",")),
                map(
                    separated_list(
                        w(tag(",")),
                        separated_pair(
                            w(one_of("tdscrux")),
                            w(tag("=")),
                            w(alt((
                                recognize(float),
                                recognize(digit1),
                                recognize(synchronicity),
                                recognize(complexity),
                                recognize(direction),
                                recognize(logical_stream_type),
                                recognize(bool),
                            ))),
                        ),
                    ),
                    |opts| opts.into_iter().collect::<HashMap<char, &str>>(),
                ),
            )),
            tag(">"),
        )),
        |(_, data, opt, _)| -> std::result::Result<LogicalStreamType, ()> {
            let throughput = PositiveReal::new(
                opt.as_ref()
                    .and_then(|opts| opts.get(&'t').map(|x| x.parse().ok()))
                    .flatten()
                    .unwrap_or(1.),
            )
            .map_err(|_| ())?;

            let dimensionality = opt
                .as_ref()
                .and_then(|opts| opts.get(&'d').map(|x| x.parse().ok()))
                .flatten()
                .unwrap_or(0);

            let synchronicity = opt
                .as_ref()
                .and_then(|opts| {
                    opts.get(&'s')
                        .map(|x| synchronicity(x).ok().map(|(_, x)| x))
                })
                .flatten()
                .unwrap_or_else(Synchronicity::default);

            let complexity = opt
                .as_ref()
                .and_then(|opts| opts.get(&'c').map(|x| complexity(x).ok().map(|(_, x)| x)))
                .flatten()
                .unwrap_or_else(Complexity::default);

            let direction = opt
                .as_ref()
                .and_then(|opts| opts.get(&'r').map(|x| direction(x).ok().map(|(_, x)| x)))
                .flatten()
                .unwrap_or_else(Direction::default);

            let user = opt
                .as_ref()
                .and_then(|opts| {
                    opts.get(&'u')
                        .map(|x| logical_stream_type(x).ok().map(|(_, x)| x))
                })
                .unwrap_or(Option::None);

            let keep = opt
                .as_ref()
                .and_then(|opts| opts.get(&'x').map(|x| bool(x).ok().map(|(_, x)| x)))
                .flatten()
                .unwrap_or(false);

            Ok(Stream::new(
                data,
                throughput,
                dimensionality,
                synchronicity,
                complexity,
                direction,
                user,
                keep,
            )
            .into())
        },
    )(input)
}

pub fn mode(input: &str) -> Result<&str, Mode> {
    map(alt((tag("in"), tag("out"))), |x: &str| x.parse().unwrap())(input)
}

pub fn interface(input: &str) -> Result<&str, Interface> {
    map(
        tuple((w(name), w(tag(":")), mode, multispace1, logical_stream_type)),
        |(n, _, m, _, t): (Name, _, Mode, _, LogicalStreamType)| Interface::new(n, m, t),
    )(input)
}

pub fn streamlet(input: &str) -> Result<&str, Streamlet> {
    map_res(
        tuple((
            w(tag("Streamlet")),
            w(name),
            w(tag("(")),
            separated_list(w(tag(",")), w(interface)),
            tag(")"),
        )),
        |(_, n, _, il, _): (_, Name, _, Vec<Interface>, _)| {
            Streamlet::from_builder(n, il.into_iter().collect())
        },
    )(input)
}

pub fn list_of_streamlets(input: &str) -> Result<&str, Vec<Streamlet>> {
    map(
        preceded(ws0, separated_list(ws1, streamlet)),
        |l: Vec<Streamlet>| l,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::UniquelyNamedBuilder;

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment("/* this is a comment */"),
            Ok(("", " this is a comment "))
        );
        assert_eq!(
            comment("/* this is a ****** / comment */"),
            Ok(("", " this is a ****** / comment "))
        );
    }

    #[test]
    fn parse_name() {
        assert_eq!(name("test"), Ok(("", Name::try_new("test").unwrap())));
        assert_eq!(
            name("test abc"),
            Ok((" abc", Name::try_new("test").unwrap()))
        );
        assert!(name("1test").is_err());
    }

    #[test]
    fn parse_bool() {
        assert_eq!(bool("true"), Ok(("", true)));
        assert_eq!(bool("false"), Ok(("", false)));
        assert!(bool("_").is_err())
    }

    #[test]
    fn parse_null() {
        assert_eq!(null("Null"), Ok(("", LogicalStreamType::Null)));
        assert!(null("null").is_err());
    }

    #[test]
    fn parse_bits() {
        assert_eq!(
            bits("Bits<3>"),
            Ok(("", LogicalStreamType::try_new_bits(3).unwrap()))
        );
    }

    #[test]
    fn parse_group() {
        assert_eq!(
            group("Group< a    :  Null ,  b:Bits<5>>"),
            Ok((
                "",
                Group::try_new(vec![
                    ("a", LogicalStreamType::Null),
                    ("b", LogicalStreamType::try_new_bits(5).unwrap())
                ])
                .unwrap()
                .into()
            ))
        );
    }

    #[test]
    fn parse_union() {
        assert_eq!(
            union("Union<a:Null,b:Bits<5>>"),
            Ok((
                "",
                Union::try_new(vec![
                    ("a", LogicalStreamType::Null),
                    ("b", LogicalStreamType::try_new_bits(5).unwrap())
                ])
                .unwrap()
                .into()
            ))
        );
    }

    #[test]
    fn parse_complexity() {
        assert_eq!(
            complexity("5.2.4.5"),
            Ok(("", Complexity::new(vec![5, 2, 4, 5]).unwrap()))
        );
    }

    #[test]
    fn parse_synchronicity() {
        assert_eq!(synchronicity("Sync"), Ok(("", Synchronicity::Sync)));
        assert_eq!(synchronicity("Desync"), Ok(("", Synchronicity::Desync)));
        assert_eq!(synchronicity("Flatten"), Ok(("", Synchronicity::Flatten)));
        assert_eq!(
            synchronicity("FlatDesync"),
            Ok(("", Synchronicity::FlatDesync))
        );
    }

    #[test]
    fn parse_direction() {
        assert_eq!(direction("Forward"), Ok(("", Direction::Forward)));
        assert_eq!(direction("Reverse"), Ok(("", Direction::Reverse)));
    }

    #[test]
    fn parse_mode() {
        assert_eq!(mode("in"), Ok(("", Mode::In)));
        assert_eq!(mode("out"), Ok(("", Mode::Out)));
    }

    #[test]
    fn parse_interface() {
        assert_eq!(
            interface("a :  in Null"),
            Ok((
                "",
                Interface::try_new("a", Mode::In, LogicalStreamType::Null).unwrap()
            ))
        );
        assert_eq!(
            interface("b:out Bits<1>"),
            Ok((
                "",
                Interface::try_new("b", Mode::Out, LogicalStreamType::try_new_bits(1).unwrap(),)
                    .unwrap()
            ))
        );
    }

    #[test]
    fn parse_stream() {
        assert_eq!(
            stream("Stream< Union< a  :  Null  ,b:Bits<1>,c:Group<d:Null,e:Null>>,t=0.01 ,d=2,c=4.2,u=Group<u0:Bits<1>,u1:Bits<2>>,x=false>"),
            Ok((
                "",
                Stream::new(
                    LogicalStreamType::try_new_union(vec![
                        ("a", LogicalStreamType::Null),
                        ("b", LogicalStreamType::try_new_bits(1).unwrap()),
                        (
                            "c",
                            LogicalStreamType::try_new_group(vec![
                                ("d", LogicalStreamType::Null),
                                ("e", LogicalStreamType::Null),
                            ])
                            .unwrap(),
                        ),
                    ])
                    .unwrap(),
                    PositiveReal::new(0.01).unwrap(),
                    2,
                    Synchronicity::default(),
                    Complexity::new(vec![4, 2]).unwrap(),
                    Direction::Forward,
                    Some(
                        LogicalStreamType::try_new_group(vec![("u0", 1), ("u1", 2)]).unwrap(),
                    ),
                    false,
                ).into()
            ))
        );
    }

    #[test]
    fn parse_streamlet() {
        assert_eq!(
            streamlet(concat!(
                "Streamlet test (\n",
                "  a : in Group< a : Bits< 1 >,\n",
                "                b : Bits< 2>\n",
                "              >,\n",
                "  c : out Null\n",
                ")",
            )),
            Ok((
                "",
                Streamlet::from_builder(
                    Name::try_new("test").unwrap(),
                    UniquelyNamedBuilder::new()
                        .with_item(
                            Interface::try_new(
                                "a",
                                Mode::In,
                                Group::try_new(vec![("a", 1), ("b", 2)]).unwrap()
                            )
                            .unwrap()
                        )
                        .with_item(
                            Interface::try_new("c", Mode::Out, LogicalStreamType::Null).unwrap()
                        )
                )
                .unwrap()
            ))
        );
    }

    fn test_streamlet(name: impl Into<String>) -> Streamlet {
        Streamlet::from_builder(
            Name::try_new(name).unwrap(),
            UniquelyNamedBuilder::new().with_items(vec![
                Interface::try_new("a", Mode::In, LogicalStreamType::Null).unwrap(),
                Interface::try_new("b", Mode::Out, LogicalStreamType::Null).unwrap(),
            ]),
        )
        .unwrap()
    }

    #[test]
    fn parse_single_streamlet() {
        assert_eq!(
            list_of_streamlets("Streamlet x ( a : in Null, b : out Null )"),
            Ok((
                "",
                UniquelyNamedBuilder::new()
                    .with_items(vec![test_streamlet("x"),])
                    .finish()
                    .unwrap()
            ))
        );
    }

    #[test]
    fn parse_list_of_streamlets() {
        assert_eq!(
            list_of_streamlets(concat!(
                "/* Some comment */\n",
                "Streamlet a ( a: in Null, b: out Null)\n",
                "/* Another comment */\n",
                "Streamlet b ( a: in Null, b: out Null)\n",
                "Streamlet c ( a: in Null, b: out Null)",
            )),
            Ok((
                "",
                UniquelyNamedBuilder::new()
                    .with_items(vec![
                        test_streamlet("a"),
                        test_streamlet("b"),
                        test_streamlet("c"),
                    ])
                    .finish()
                    .unwrap()
            ))
        );
    }
}
