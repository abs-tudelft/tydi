//! Parser methods and implementations for Tydi types and formats.
//!
//! The parser module is enabled by the `parser` feature flag. It adds some
//! utitity parser methods and implementations of parsers for Tydi and
//! streamlet types.
//!
//! The current parsers are built using [`nom`], a parser combinators crate.
//!
//! [`nom`]: https://crates.io/crates/nom
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1, space0},
    combinator::map_res,
    multi::separated_nonempty_list,
    sequence::{delimited, preceded, terminated},
    IResult,
};

pub mod data;
pub mod river;
pub mod streamlet;

/// Returns a parser function to parse a Type<_> using the provided `inner`
/// parser.
pub(crate) fn r#type<'a, T, F>(name: &'a str, inner: F) -> impl Fn(&'a str) -> IResult<&'a str, T>
where
    F: Fn(&'a str) -> IResult<&'a str, T>,
{
    preceded(tag(name), delimited(char('<'), inner, char('>')))
}

/// Returns a parser function which allow space characters after the provided
/// `inner` parser.
pub(crate) fn space_opt<'a, T, F>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, T>
where
    F: Fn(&'a str) -> IResult<&'a str, T>,
{
    terminated(inner, space0)
}

/// Returns a parser function to parse non-empty comma-separated,
/// space-optional lists.
pub(crate) fn nonempty_comma_list<'a, T, F>(
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<T>>
where
    F: Fn(&'a str) -> IResult<&'a str, T>,
{
    separated_nonempty_list(space_opt(char(',')), inner)
}

/// Parses some input digits to a usize.
pub(crate) fn usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}
