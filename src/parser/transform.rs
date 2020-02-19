use crate::logical::{Direction, Group, Union};
use crate::logical::{LogicalStreamType, Stream, Synchronicity};
use crate::parser::Rule;
use crate::physical::Complexity;
use crate::streamlet::{Interface, Mode, Streamlet, StreamletBuilder};
use crate::Name;
use crate::{NonNegative, PositiveReal};
use pest::iterators::{Pair, Pairs};
use std::convert::{Infallible, TryFrom, TryInto};
use std::fmt::{Display, Error, Formatter};

fn check_rule<T>(
    pair: Pair<Rule>,
    rule: Rule,
    f: impl Fn(Pair<Rule>) -> Result<T, TransformError>,
) -> Result<T, TransformError> {
    if pair.as_rule() == rule {
        f(pair)
    } else {
        Err(TransformError::BadRule(format!(
            "Expected: \"{:?}\", Actual: \"{:?}\"",
            rule, pair
        )))
    }
}

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
        check_rule(pair, Rule::compl, |pair| {
            pair.as_str()
                .parse::<Complexity>()
                .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Synchronicity {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::synchronicity, |pair| {
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
        check_rule(pair, Rule::dir, |pair| {
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
        check_rule(pair, Rule::mode, |pair| match pair.as_str() {
            "in" => Ok(Mode::In),
            "out" => Ok(Mode::Out),
            _ => unreachable!(),
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for StreamOptList {
    type Error = TransformError;
    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::stream_opt_list, |pair| {
            let mut stream_opt_list = StreamOptList::default();

            let pairs = list_flatten(pair).into_iter().map(|stream_opt| {
                stream_opt
                    .into_inner()
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
        check_rule(pair, Rule::typ, |pair| {
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
                Rule::union => {
                    let union: Union = pair.try_into()?;
                    Ok(union.into())
                }
                _ => unreachable!(),
            }
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for PositiveReal {
    type Error = TransformError;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::float, |pair| {
            PositiveReal::new(
                pair.as_str()
                    .trim()
                    .parse::<f64>()
                    .map_err(|e| TransformError::BadArgument(e.to_string()))?,
            )
            .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Name {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::ident, |pair| {
            pair.as_str()
                .trim()
                .parse::<Name>()
                .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Streamlet {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::streamlet, |pair| {
            let mut pairs = pair.into_inner();
            let name: Name = pairs.next().unwrap().try_into()?;
            let mut builder = StreamletBuilder::new(name);
            for interface in pairs {
                builder.add_interface(interface.try_into()?);
            }
            Ok(builder
                .finish()
                .map_err(|e| TransformError::BadArgument(e.to_string()))?)
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Stream {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::stream, |pair| {
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

fn list_flatten(pair: Pair<Rule>) -> Vec<Pair<Rule>> {
    // todo(mb): return type
    match pair.as_rule() {
        Rule::field_list | Rule::stream_opt_list => {
            let mut fields = pair.into_inner();
            let pairs = Pairs::single(fields.next().unwrap());
            if let Some(field_list) = fields.next() {
                pairs.chain(list_flatten(field_list)).collect::<Vec<_>>()
            } else {
                pairs.collect::<Vec<_>>()
            }
        }
        _ => panic!("field_list rules only"),
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Group {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::group, |pair| {
            Group::try_new(
                list_flatten(pair.into_inner().next().unwrap())
                    .into_iter()
                    .map(|field| {
                        let mut pairs = field.into_inner();
                        (pairs.next().unwrap(), pairs.next().unwrap())
                    }),
            )
            .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

// TODO(johanpel): create a tryfrom apparaat for field list
impl<'i> TryFrom<Pair<'i, Rule>> for Union {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::union, |pair| {
            Union::try_new(
                list_flatten(pair.into_inner().next().unwrap())
                    .into_iter()
                    .map(|field| {
                        let mut pairs = field.into_inner();
                        (pairs.next().unwrap(), pairs.next().unwrap())
                    }),
            )
            .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Interface {
    type Error = TransformError;

    fn try_from(pair: Pair<'i, Rule>) -> Result<Self, Self::Error> {
        check_rule(pair, Rule::interface, |pair| {
            let mut pairs = pair.into_inner();
            Interface::try_new(
                pairs.next().unwrap(),
                pairs.next().unwrap().try_into()?,
                pairs.next().unwrap(),
            )
            .map_err(|e| TransformError::BadArgument(e.to_string()))
        })
    }
}

fn transform<T, U, E>(value: impl Iterator<Item = U>) -> Result<T, TransformError>
where
    T: TryFrom<U, Error = E>,
    E: Into<TransformError>,
    U: std::fmt::Debug,
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
    check_rule(pair, Rule::bool, |pair| {
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
    check_rule(pair, Rule::uint, |pair| {
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
        transform_ok!(compl, "4.1.3", Complexity::new(vec![4, 1, 3]).unwrap());
    }

    #[test]
    fn test_synchronicity() {
        transform_ok!(synchronicity, "Sync", Synchronicity::Sync);
        transform_ok!(synchronicity, "Flatten", Synchronicity::Flatten);
        transform_ok!(synchronicity, "Desync", Synchronicity::Desync);
        transform_ok!(synchronicity, "FlatDesync", Synchronicity::FlatDesync);
    }

    #[test]
    fn test_direction() {
        transform_ok!(dir, "Forward", Direction::Forward);
        transform_ok!(dir, "Reverse", Direction::Reverse);
    }

    #[test]
    fn test_mode() {
        transform_ok!(mode, "in", Mode::In);
        transform_ok!(mode, "out", Mode::Out);
    }

    #[test]
    fn test_float() {
        transform_ok!(float, "0.1", PositiveReal::new(0.1).unwrap());
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

        let e2 = Stream::new(
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
        );

        transform_ok!(stream,
        "Stream<Union<a: Null, b: Bits<1>, c: Group<d:Null, e:Null>>,t=0.01,d=2,c=4.2,u=Group<u0:Bits<1>,u1:Bits<2>>,x=false>",
        e2);
    }

    #[test]
    fn test_interface() {
        transform_ok!(
            interface,
            "a : in Bits<1>;",
            Interface::try_new("a", Mode::In, LogicalStreamType::try_new_bits(1).unwrap()).unwrap()
        );
    }

    #[test]
    fn test_streamlet() -> Result<(), Box<dyn std::error::Error>> {
        transform_ok!(
            streamlet,
            "Streamlet test { a: in Group<a:Bits<1>, b:Bits<2>>; c: out Null; }",
            StreamletBuilder::new(Name::try_new("test").unwrap())
                .with_interface(Interface::new(
                    "a".try_into()?,
                    Mode::In,
                    Group::try_new(vec![("a", 1), ("b", 2)]).unwrap()
                ))
                .with_interface(
                    Interface::try_new("c", Mode::Out, LogicalStreamType::Null).unwrap()
                )
                .finish()
                .unwrap()
        );
        Ok(())
    }
}
