use crate::logical::Direction;
use crate::logical::LogicalStreamType;
use crate::logical::Synchronicity;
use crate::logical::{Group, Stream, Union};
use crate::physical::Complexity;
use crate::{Name, Positive, PositiveReal};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::bytes::complete::take_while;
use nom::character::complete::digit1;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::separated_list;
use nom::number::complete::float;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::tuple;
use std::collections::HashMap;

// #[derive(Debug, PartialEq)]
// pub struct ParserError<I> {
//     kind: ErrorKind<I>,
//     backtrace: Vec<ErrorKind<I>>,
// }

// impl<I> ParserError<I> {
//     fn backtrace(&mut self, other: ParserError<I>) {
//         self.backtrace.push(other.kind);
//     }
// }

// #[derive(Debug, PartialEq)]
// enum ErrorKind<I> {
//     Nom(I, nom::error::ErrorKind),
//     BadName(String),
// }

// impl<I> ParseError<I> for ParserError<I> {
//     fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
//         ParserError {
//             kind: ErrorKind::Nom(input, kind),
//             backtrace: vec![],
//         }
//     }

//     fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
//         other.backtrace(Self::from_error_kind(input, kind));
//         other
//     }
// }

// todo(mb): whitespace tollerance

type Result<I, T> = nom::IResult<I, T, nom::error::VerboseError<I>>;

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
    map_res(delimited(tag("Bits<"), digit1, tag(">")), |x: &str| {
        LogicalStreamType::try_new_bits(x.parse().unwrap()).map_err(|_| ())
    })(input)
}

pub fn logical_stream_type(input: &str) -> Result<&str, LogicalStreamType> {
    alt((null, bits, group, union, stream))(input)
}

fn fields(input: &str) -> Result<&str, Vec<(Name, LogicalStreamType)>> {
    separated_list(
        tag(","),
        separated_pair(name, tag(":"), logical_stream_type),
    )(input)
}

pub fn group(input: &str) -> Result<&str, LogicalStreamType> {
    map_res(
        delimited(tag("Group<"), fields, tag(">")),
        |fields: Vec<(Name, LogicalStreamType)>| {
            Group::try_new(fields).map(Into::into).map_err(|_| ())
        },
    )(input)
}

pub fn union(input: &str) -> Result<&str, LogicalStreamType> {
    map_res(
        delimited(tag("Union<"), fields, tag(">")),
        |fields: Vec<(Name, LogicalStreamType)>| {
            Union::try_new(fields).map(Into::into).map_err(|_| ())
        },
    )(input)
}

pub fn complexity(input: &str) -> Result<&str, Complexity> {
    map_res(separated_list(tag("."), digit1), |level: Vec<&str>| {
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
    dbg!(input);
    map_res(
        tuple((
            tag("Stream<"),
            logical_stream_type,
            opt(preceded(
                tag(","),
                map(
                    separated_list(
                        tag(","),
                        separated_pair(
                            one_of("tdscrux"),
                            tag("="),
                            alt((
                                recognize(float),
                                recognize(digit1),
                                recognize(synchronicity),
                                recognize(complexity),
                                recognize(direction),
                                recognize(logical_stream_type),
                                recognize(bool),
                            )),
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

            dbg!(&data);
            dbg!(&opt);
            Ok(Stream::new(
                data,
                throughput,
                dimensionality,
                synchronicity,
                complexity,
                direction,
                user.map(Box::new),
                keep,
            )
            .into())
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Ok(("", LogicalStreamType::Bits(Positive::new(3).unwrap())))
        );
    }

    #[test]
    fn parse_group() {
        assert_eq!(
            group("Group<a:Null,b:Bits<5>>"),
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
    fn parse_complexity() {
        assert_eq!(
            complexity("5.2.4.5"),
            Ok(("", Complexity::new(vec![5, 2, 4, 5]).unwrap()))
        );
    }

    #[test]
    fn parse_synchronicity() {
        assert_eq!(synchronicity("Flatten"), Ok(("", Synchronicity::Flatten)));
    }

    #[test]
    fn parse_direction() {
        assert_eq!(direction("Forward"), Ok(("", Direction::Forward)));
    }

    #[test]
    fn parse_stream() {
        assert_eq!(
            stream("Stream<Union<a:Null,b:Bits<1>,c:Group<d:Null,e:Null>>,t=0.01,d=2,c=4.2,u=Group<u0:Bits<1>,u1:Bits<2>>,x=false>"),
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
                    Some(Box::new(
                        LogicalStreamType::try_new_group(vec![("u0", 1), ("u1", 2)]).unwrap(),
                    )),
                    false,
                ).into()
            ))
        );
    }
}
