//! Parser methods and implementations for Tydi types.
//!
//! The parser module is enabled by the `parser` feature flag. It adds some
//! utitity parser methods and implementations of parsers for Tydi stream and
//! streamlet types.
//!
//! The current parsers are built using [`nom`], a parser combinators crate.
//!
//! [`nom`]: https://crates.io/crates/nom
use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, digit1, space0},
    combinator::{map_res, opt},
    multi::separated_nonempty_list,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[cfg(feature = "data")]
pub mod data;

pub mod logical;
pub mod streamlet;

/// Parses an identifier.
fn identifier(input: &str) -> IResult<&str, String> {
    alphanumeric1(input).map(|(a, b)| (a, b.to_string()))
}

/// Returns a parser function to parse a Type<_> using the provided `inner`
/// parser. This includes an optional identifier.
pub(crate) fn r#type<'a, T, F>(
    name: &'a str,
    inner: F,
) -> impl Fn(&'a str) -> IResult<&'a str, (Option<String>, T)>
where
    F: Fn(&'a str) -> IResult<&'a str, T>,
{
    tuple((
        opt(terminated(identifier, space_opt(char(':')))),
        preceded(tag(name), delimited(char('<'), inner, char('>'))),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier() {
        assert_eq!(identifier("asdf"), Ok(("", "asdf".to_string())));
        assert_eq!(identifier("asdf asdf"), Ok((" asdf", "asdf".to_string())));
        assert_eq!(identifier("1234 asdf"), Ok((" asdf", "1234".to_string())));
    }

    #[test]
    fn parse_type() {
        let test = r#type("Test", identifier);
        assert_eq!(test("Test<a>").unwrap(), ("", (None, "a".to_string())));
    }
}
