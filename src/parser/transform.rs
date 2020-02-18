use crate::generator::common::Mode;
use crate::logical::Direction;
use crate::logical::{LogicalStreamType, Stream, Synchronicity};
use crate::parser::Rule;
use crate::physical::Complexity;
use crate::streamlet::Streamlet;
use crate::Name;
use crate::{NonNegative, PositiveReal};
use indexmap::IndexMap;
use pest::iterators::Pair;
use std::convert::{Infallible, TryFrom, TryInto};
use std::fmt::{Display, Error, Formatter};

#[derive(Debug, PartialEq)]
pub enum TransformError {
    NoMatch,
    DuplicateArguments,
    Complexity,
    Synchronicity,
    MissingArgument,
    BadArgument,
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
        match pair.as_rule() {
            Rule::compl => pair
                .as_str()
                .parse::<Complexity>()
                .map_err(|_| TransformError::BadArgument),
            _ => Err(TransformError::Complexity),
        }
    }
}

impl TryFrom<Rule> for Synchronicity {
    type Error = TransformError;

    fn try_from(value: Rule) -> Result<Self, Self::Error> {
        match value {
            Rule::sync => Ok(Synchronicity::Sync),
            Rule::flat => Ok(Synchronicity::Flatten),
            Rule::desync => Ok(Synchronicity::Desync),
            Rule::flatdesync => Ok(Synchronicity::FlatDesync),
            _ => Err(TransformError::NoMatch),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Synchronicity {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream_opt_dir => pair.into_inner().next().unwrap().as_rule().try_into(),
            _ => Err(TransformError::Synchronicity),
        }
    }
}

impl TryFrom<Rule> for Direction {
    type Error = TransformError;

    fn try_from(value: Rule) -> Result<Self, Self::Error> {
        match value {
            Rule::forward => Ok(Direction::Forward),
            Rule::reverse => Ok(Direction::Reverse),
            _ => Err(TransformError::NoMatch),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Direction {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream_opt_dir => pair.into_inner().next().unwrap().as_rule().try_into(),
            _ => Err(TransformError::Synchronicity),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Mode {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::mode => match pair.as_str() {
                "in" => Ok(Mode::In),
                "out" => Ok(Mode::Out),
                _ => unreachable!(),
            },
            _ => Err(TransformError::NoMatch),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for LogicalStreamType {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::typ => {
                let pair = pair.into_inner().next().unwrap();
                match pair.as_rule() {
                    Rule::null => Ok(LogicalStreamType::Null),
                    Rule::bits => pair.into_inner().next().unwrap().try_into(),
                    _ => Err(TransformError::NoMatch),
                }
            }
            _ => Err(TransformError::NoMatch),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for PositiveReal {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream_opt_tput => PositiveReal::new(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse::<f64>()
                    .unwrap(),
            )
            .map_err(|_| TransformError::BadArgument),
            _ => Err(TransformError::NoMatch),
        }
    }
}

fn transform<'i, T, U, E>(value: impl Iterator<Item = U>) -> Result<T, TransformError>
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

impl<'i> TryFrom<Pair<'i, Rule>> for Streamlet {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
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
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Stream {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream => {
                let mut pairs = pair.into_inner();
                let typ: LogicalStreamType = pairs
                    .next()
                    .ok_or_else(|| TransformError::MissingArgument)?
                    .try_into()?;

                let complexity: Complexity = transform(pairs.clone()).unwrap_or_default();

                Ok(Stream::new(
                    typ,
                    transform(pairs.clone()).unwrap_or(PositiveReal::new(1.0).unwrap()),
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
            }
            _ => Err(TransformError::NoMatch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SDFParser;
    use pest::Parser;

    #[test]
    fn test_complexity() {
        let parse_result = SDFParser::parse(Rule::compl, "4.1.3");
        match parse_result {
            Ok(mut pairs_iter) => {
                if let Some(pair) = pairs_iter.next() {
                    match pair.as_str().parse::<Complexity>() {
                        Ok(c) => assert_eq!(c, Complexity::new(vec![4, 1, 3]).unwrap()),
                        Err(_) => panic!(),
                    }
                }
            }
            Err(_) => panic!(),
        }
    }
}
