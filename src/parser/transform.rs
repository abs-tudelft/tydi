use crate::logical::{LogicalStreamType, Stream, Synchronicity};
use crate::parser::Rule;
use crate::physical::Complexity;
use crate::{NonNegative, PositiveReal};
use pest::iterators::Pair;
use std::convert::{Infallible, TryFrom, TryInto};
use std::fmt::{Display, Error, Formatter};

#[derive(Debug)]
pub enum TransformError {
    NoMatch,
    DuplicateArguments,
    Complexity,
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

impl TryFrom<Rule> for Synchronicity {
    type Error = TransformError;

    fn try_from(value: Rule) -> Result<Self, Self::Error> {
        match value {
            Rule::sync =>
            Rule::flat =>
            Rule::desync =>
                Rule::flatdesync =>
            _
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Synchronicity {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream_opt_sync => pair
                .into_inner()
                .next()
                .unwrap()
                .as_rule()
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
            _ => Err(Box::new(TransformError::NoMatch)),
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

impl<'i> TryFrom<Pair<'i, Rule>> for Stream {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::stream => {
                let mut pairs = pair.into_inner();
                let typ: LogicalStreamType = pairs
                    .next()
                    .ok_or_else(|| TransformError::MissingArgument)?
                    .try_into()?;
                let throughput: PositiveReal =
                    transform(pairs.clone()).unwrap_or(PositiveReal::new(1.0).unwrap());
                let dimensionality: NonNegative = transform_dim(pairs.clone()).unwrap_or_default();
                let synchronicity: Synchronicity = transform(pairs.clone()).unwrap_or_default();

                //                let complexity: Complexity = pairs.try_into().unwrap_or_default();
                //                let direction: Direction = pairs.try_into().unwrap_or_default();
                //                let user: Option<Box<LogicalStreamType>> = pairs.try_into().unwrap_or(None);
                //                let keep: bool = pair.try_into().unwrap_or(false);
                //                Ok(Stream::new(
                //                    typ,
                //                    throughput,
                //                    dimensionality,
                //                    synchronicity,
                //                    complexity,
                //                    direction,
                //                    user,
                //                    keep,
                //                ))
            }
            _ => unimplemented!(),
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
