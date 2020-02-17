use crate::streamlet::Streamlet;
use pest::Parser;

use crate::logical::{Direction, LogicalStreamType, Stream, Synchronicity};
use crate::parser::{Rule, SDFParser};
use crate::physical::Complexity;
use crate::{NonNegative, PositiveReal};
use pest::iterators::{Pair, Pairs};
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Error, Formatter};

#[derive(Debug)]
pub enum TransformError {
    Meh,
    Complexity,
}

impl std::error::Error for TransformError {}

impl Display for TransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Complexity {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::compl => pair
                .as_str()
                .parse::<Complexity>()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
            _ => Err(Box::new(TransformError::Complexity)),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for LogicalStreamType {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::null => Ok(LogicalStreamType::Null),
            _ => Err(Box::new(TransformError::Meh)),
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Stream {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pair: Pair<'i, _>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream => {
                let mut pairs = pair.into_inner();
                let typ: LogicalStreamType = pairs.next().try_into()?;

                let throughput: PositiveReal = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or(PositiveReal::new(1.0).unwrap());
                let dimensionality: NonNegative = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or_default();
                let synchronicity: Synchronicity = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or_default();
                let complexity: Complexity = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or_default();
                let direction: Direction = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or_default();
                let user: Option<Box<LogicalStreamType>> = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or(None);
                let keep: bool = pair
                    .into_inner()
                    .skip(1)
                    .filter_map(|p| p.try_into().ok())
                    .unwrap_or(false);
                Stream::new(
                    typ,
                    PositiveReal::new(1.).unwrap(),
                    0,
                    Synchronicity::Sync,
                    complexity,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
