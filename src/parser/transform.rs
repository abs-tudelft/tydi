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

// TODO(johanpel): upgrade error management.
#[derive(Debug, PartialEq)]
pub enum TransformError {
    BadRule(String),
    NoMatch,
    DuplicateArguments,
    MissingArgument,
    BadArgument(String),
}

struct StreamOptList {
    throughput: PositiveReal,
    dimensionality: NonNegative,
    synchronicity: Synchronicity,
    complexity: Complexity,
    direction: Direction,
    user: Option<LogicalStreamType>,
    keep: bool,
}

impl StreamOptList {
    fn set_throughput(&mut self, value: Result<PositiveReal, TransformError>) {
        value.map(|x| self.throughput = x).ok();
    }
    fn set_dimensionality(&mut self, value: Result<NonNegative, TransformError>) {
        value.map(|x| self.dimensionality = x).ok();
    }
    fn set_synchronicity(&mut self, value: Result<Synchronicity, TransformError>) {
        value.map(|x| self.synchronicity = x).ok();
    }
    fn set_complexity(&mut self, value: Result<Complexity, TransformError>) {
        value.map(|x| self.complexity = x).ok();
    }
    fn set_direction(&mut self, value: Result<Direction, TransformError>) {
        value.map(|x| self.direction = x).ok();
    }
    fn set_user(&mut self, value: Result<Option<LogicalStreamType>, TransformError>) {
        value.map(|x| self.user = x).ok();
    }
    fn set_keep(&mut self, value: Result<bool, TransformError>) {
        value.map(|x| self.keep = x).ok();
    }
}

impl Default for StreamOptList {
    fn default() -> Self {
        StreamOptList {
            throughput: PositiveReal::new(1.0).unwrap(),
            dimensionality: 0,
            synchronicity: Synchronicity::default(),
            complexity: Complexity::default(),
            direction: Direction::default(),
            user: None,
            keep: false,
        }
    }
}

macro_rules! check_rule {
    ($pair:ident, $expected_rule:ident, $code:block) => {
        match $pair.as_rule() {
            Rule::$expected_rule => $code,
            _ => Err(TransformError::BadRule(format!(
                "Expected: \"{}\", Actual: \"{:?}\"",
                stringify!($expected_rule),
                $pair
            ))),
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
        check_rule!(pair, compl, {
            pair.as_str()
                .parse::<Complexity>()
                .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Synchronicity {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, synchronicity, {
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
        check_rule!(pair, dir, {
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

impl<'i> TryFrom<Pair<'i, Rule>> for StreamOptList {
    type Error = TransformError;
    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule!(pair, stream_opt_list, {
            let mut stream_opt_list = StreamOptList::default();

            let pairs = pair
                .into_inner()
                .flatten()
                .filter(|pair| pair.as_rule() == Rule::stream_opt)
                .map(|pair| {
                    pair.into_inner()
                        .next()
                        .unwrap()
                        .into_inner()
                        .next()
                        .unwrap()
                });

            stream_opt_list.set_throughput(transform(pairs.clone()));
            stream_opt_list.set_dimensionality(transform_arguments(pairs.clone(), |pair| {
                transform_uint(pair).ok()
            }));
            stream_opt_list.set_synchronicity(transform(pairs.clone()));
            stream_opt_list.set_complexity(transform(pairs.clone()));
            stream_opt_list.set_direction(transform(pairs.clone()));
            stream_opt_list.set_user(transform(pairs.clone()).map(Option::Some));
            stream_opt_list.set_keep(transform_arguments(pairs.clone(), |pair| {
                transform_bool(pair).ok()
            }));
            Ok(stream_opt_list)
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
                Rule::bits => {
                    let width: NonNegative = transform_uint(pair.into_inner().next().unwrap())?;
                    let bits: LogicalStreamType = LogicalStreamType::try_new_bits(width)
                        .map_err(|e| TransformError::BadArgument(e.to_string()))?;
                    Ok(bits)
                }
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
        check_rule!(pair, float, {
            PositiveReal::new(pair.as_str().parse::<f64>().unwrap())
                .map_err(|e| TransformError::BadArgument(e.to_string()))
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
                        .map_err(|e| TransformError::BadArgument(e.to_string()))?;
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

            let stream_opt_list = match pairs.next() {
                Some(iter) => iter.try_into()?,
                None => StreamOptList::default(),
            };

            Ok(Stream::new(
                typ,
                stream_opt_list.throughput,
                stream_opt_list.dimensionality,
                stream_opt_list.synchronicity,
                stream_opt_list.complexity,
                stream_opt_list.direction,
                stream_opt_list.user.map(Box::new),
                stream_opt_list.keep,
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
            .map_err(|e| TransformError::BadArgument(e.to_string()))
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

fn transform_bool(pair: Pair<Rule>) -> Result<bool, TransformError> {
    check_rule!(pair, bool, {
        pair.as_str()
            .parse::<bool>()
            .map_err(|e| TransformError::BadArgument(e.to_string()))
    })
}

fn transform_arguments<'i, T>(
    pairs: impl Iterator<Item = Pair<'i, Rule>>,
    transformer: impl FnMut(Pair<Rule>) -> Option<T>,
) -> Result<T, TransformError> {
    pairs
        .filter_map(transformer)
        .next()
        .ok_or_else(|| TransformError::MissingArgument)
}

fn transform_uint(pair: Pair<Rule>) -> Result<NonNegative, TransformError> {
    check_rule!(pair, uint, {
        pair.as_str()
            .parse::<NonNegative>()
            .map_err(|e| TransformError::BadArgument(e.to_string()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SDFParser;
    use pest::Parser;

    macro_rules! transform_ok {
        ($rule:ident, $string:expr, $expected:ident) => {
            let parse = SDFParser::parse(Rule::$rule, $string);
            assert!(parse.is_ok());
            let pair = parse.unwrap().next();
            assert!(pair.is_some());
            let result = pair.unwrap().try_into();
            assert_eq!(result, Ok($expected)); // compare with expected
        };
        ($rule:ident, $string:expr, $expected:expr) => {
            let temp = $expected;
            transform_ok!($rule, $string, temp)
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
    fn test_dimensionality() {
        unimplemented!()
    }

    #[test]
    fn test_stream() {
        let e0 = Stream::new(
            LogicalStreamType::try_new_bits(1).unwrap(),
            PositiveReal::new(1.).unwrap(),
            0,
            Synchronicity::Sync,
            Complexity::default(),
            Direction::Forward,
            None,
            false,
        );
        transform_ok!(stream, "Stream<Bits<1>>", e0);
    }

    #[test]
    fn test_stream1() {
        let e1 = Stream::new(
            LogicalStreamType::try_new_bits(4).unwrap(),
            PositiveReal::new(0.5).unwrap(),
            2,
            Synchronicity::FlatDesync,
            Complexity::new(vec![1, 3, 3, 7]).unwrap(),
            Direction::Reverse,
            Some(Box::new(LogicalStreamType::try_new_bits(5).unwrap())),
            true,
        );

        transform_ok!(
            stream,
            "Stream<Bits<4>, t=0.5, d=2, s=FlatDesync, c=1.3.3.7, r=Reverse, u=Bits<5>, x=true>",
            e1
        );
    }

    #[test]
    fn test_interface() {}
}
