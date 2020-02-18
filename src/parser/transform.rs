use crate::generator::common::Mode;
use crate::logical::{Direction, Group};
use crate::logical::{LogicalStreamType, Stream, Synchronicity};
use crate::parser::Rule;
use crate::physical::Complexity;
use crate::streamlet::Streamlet;
use crate::Name;
use crate::{NonNegative, PositiveReal};
use pest::iterators::Pair;
use std::convert::{Infallible, TryFrom, TryInto};
use std::fmt::{Display, Error, Formatter};

#[derive(Debug, PartialEq)]
pub enum TransformError {
    BadRule(String),
    NoMatch,
    DuplicateArguments,
    MissingArgument,
    BadArgument,
}

macro_rules! check_rule {
    ($pair:ident, $expected_rule:ident, $code:block) => {
        match $pair.as_rule() {
            Rule::$expected_rule => $code,
            _ => Err(TransformError::BadRule(
                stringify!($expected_rule).to_string(),
            )),
        }
    };
}

impl std::error::Error for TransformError {}

impl From<std::convert::Infallible> for TransformError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Display for TransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Complexity {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, stream_opt_compl, {
            pair.into_inner()
                .next()
                .unwrap()
                .as_str()
                .parse::<Complexity>()
                .map_err(|_| TransformError::BadArgument)
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Synchronicity {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, stream_opt_sync, {
            match pair.into_inner().next().unwrap().as_rule() {
                Rule::sync => Ok(Synchronicity::Sync),
                Rule::flat => Ok(Synchronicity::Flatten),
                Rule::desync => Ok(Synchronicity::Desync),
                Rule::flatdesync => Ok(Synchronicity::FlatDesync),
                _ => unreachable!(),
            }
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Direction {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, stream_opt_dir, {
            match pair.into_inner().next().unwrap().as_rule() {
                Rule::forward => Ok(Direction::Forward),
                Rule::reverse => Ok(Direction::Reverse),
                _ => unreachable!(),
            }
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Mode {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, mode, {
            match pair.as_str() {
                "in" => Ok(Mode::In),
                "out" => Ok(Mode::Out),
                _ => unreachable!(),
            }
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for LogicalStreamType {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, typ, {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::null => Ok(LogicalStreamType::Null),
                Rule::bits => pair.into_inner().next().unwrap().try_into(),
                Rule::stream => {
                    let stream: Stream = pair.try_into()?;
                    Ok(stream.into())
                }
                Rule::group => {
                    let group: Group = pair.try_into()?;
                    Ok(group.into())
                }
                _ => unreachable!(),
            }
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for PositiveReal {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, stream_opt_tput, {
            PositiveReal::new(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse::<f64>()
                    .unwrap(),
            )
            .map_err(|_| TransformError::BadArgument)
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Streamlet {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, streamlet, {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str();
            let if_list = pairs.next().unwrap().into_inner();
            let (input, output): (Vec<_>, Vec<_>) = if_list
                .map(|interface| {
                    let mut interface = interface.into_inner();
                    let name: Name = interface
                        .next()
                        .unwrap()
                        .as_str()
                        .parse::<Name>()
                        .map_err(|_| TransformError::BadArgument)?;
                    let mode: Mode = interface.next().unwrap().try_into()?;
                    let typ: LogicalStreamType = interface.next().unwrap().try_into()?;
                    Ok((name, mode, typ))
                })
                .collect::<Result<Vec<_>, TransformError>>()?
                .into_iter()
                .partition(|(_, mode, _)| mode == &Mode::In);
            Ok(Streamlet::new(
                name,
                input.into_iter().map(|(name, _, stream)| (name, stream)),
                output.into_iter().map(|(name, _, stream)| (name, stream)),
            ))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Stream {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, stream, {
            let mut pairs = pair.into_inner();
            let typ: LogicalStreamType = pairs
                .next()
                .ok_or_else(|| TransformError::MissingArgument)?
                .try_into()?;

            let complexity: Complexity = transform(pairs.clone()).unwrap_or_default();

            Ok(Stream::new(
                typ,
                transform(pairs.clone()).unwrap_or_else(|_| PositiveReal::new(1.0).unwrap()),
                transform_dim(pairs.clone()).unwrap_or_default(),
                transform(pairs.clone()).unwrap_or_default(),
                complexity,
                transform(pairs.clone()).unwrap_or_default(),
                transform(pairs.clone().skip(1))
                    .map(Box::new)
                    .map(Option::Some)
                    .unwrap_or(None),
                transform_bool(pairs.clone()).unwrap_or(false),
            ))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Group {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, group, {
            // Obtain the field list AST node
            let field_list = pair.into_inner().next().unwrap().into_inner();
            Group::try_new(
                field_list
                    .map(|field_pair| {
                        let mut field = field_pair.into_inner();
                        let field_name = field.next().unwrap().as_str();
                        let field_type: LogicalStreamType = field.next().unwrap().try_into()?;
                        Ok((field_name, field_type))
                    })
                    .collect::<Result<Vec<_>, TransformError>>()?,
            )
            .map_err(|_| TransformError::BadArgument)
        })
    }
}

fn transform<T, U, E>(value: impl Iterator<Item = U>) -> Result<T, TransformError>
where
    T: TryFrom<U, Error = E>,
    E: Into<TransformError>,
{
    let mut filtered = value.filter_map(|p| p.try_into().ok());
    // Attempt to pop a result.
    let result = filtered.next();
    // If there are more than one result, there are duplicate pairs.
    if filtered.count() > 0 {
        Err(TransformError::DuplicateArguments)
    } else {
        result.ok_or_else(|| TransformError::NoMatch)
    }
}

fn transform_bool<'i>(value: impl Iterator<Item = Pair<'i, Rule>>) -> Result<bool, TransformError> {
    value
        .filter_map(|p| {
            if p.as_rule() == Rule::stream_opt_extra {
                p.into_inner().next().unwrap().as_str().parse::<bool>().ok()
            } else {
                None
            }
        })
        .next()
        .ok_or_else(|| TransformError::MissingArgument)
}

fn transform_dim<'i>(
    value: impl Iterator<Item = Pair<'i, Rule>>,
) -> Result<NonNegative, TransformError> {
    value
        .filter_map(|p| {
            if p.as_rule() == Rule::stream_opt_dim {
                p.into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse::<NonNegative>()
                    .ok()
            } else {
                None
            }
        })
        .next()
        .ok_or_else(|| TransformError::MissingArgument)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SDFParser;
    use pest::Parser;

    macro_rules! transform_ok {
        ($rule:ident, $string:literal, $expected:expr) => {
            let parse = SDFParser::parse(Rule::$rule, $string);
            if !parse.is_ok() {
                println!("{:#?}", parse);
            }
            let pair = parse.unwrap().next();
            if pair.is_none() {
                println!("{:#?}", pair);
            }
            let result = pair.unwrap().try_into();
            if !result.is_ok() {
                println!("{:#?}", result);
            }
            assert_eq!($expected, result.unwrap()); // compare with expected
        };
    }

    #[test]
    fn test_complexity() {
        transform_ok!(
            stream_opt_compl,
            "c=4.1.3",
            Complexity::new(vec![4, 1, 3]).unwrap()
        );
    }

    #[test]
    fn test_synchronicity() {
        transform_ok!(stream_opt_sync, "s=Sync", Synchronicity::Sync);
        transform_ok!(stream_opt_sync, "s= Flatten", Synchronicity::Flatten);
        transform_ok!(stream_opt_sync, "s = Desync", Synchronicity::Desync);
        transform_ok!(stream_opt_sync, "s =FlatDesync", Synchronicity::FlatDesync);
    }

    #[test]
    fn test_direction() {
        transform_ok!(stream_opt_dir, "r =Forward", Direction::Forward);
        transform_ok!(stream_opt_dir, "r= Reverse", Direction::Reverse);
    }

    #[test]
    fn test_mode() {
        transform_ok!(mode, "in", Mode::In);
        transform_ok!(mode, "out", Mode::Out);
    }

    #[test]
    fn test_throughput() {
        transform_ok!(stream_opt_tput, "t=0.1", PositiveReal::new(0.1).unwrap());
    }

    #[test]
    fn test_stream() {
        let expected = Stream::new(
            LogicalStreamType::try_new_bits(1).unwrap(),
            PositiveReal::new(1.).unwrap(),
            0,
            Synchronicity::Sync,
            Complexity::default(),
            Direction::Forward,
            None,
            false,
        );
        transform_ok!(stream, "Stream<Bits<1>>", expected);
    }
}
