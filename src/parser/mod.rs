//! Parser methods and implementations for Tydi types.
//!
//! The parser module is enabled by the `parser` feature flag. It adds some
//! utitity parser methods and implementations of parsers for Tydi stream and
//! streamlet types.
//!
//! The current parsers are built using [`pest`].
//!
//! [`pest`]: https://crates.io/crates/pest

//#[macro_use]
//extern crate pest_derive;

use crate::parser::transform::TransformError;
use crate::streamlet::Streamlet;
use pest::Parser;
use std::convert::TryFrom;

#[derive(Parser)]
#[grammar = "parser/sdf.pest"]
pub struct SDFParser;

pub mod nom;

mod transform;

pub fn parse_streamlet(input: &str) -> Result<Streamlet, TransformError> {
    let pair = SDFParser::parse(Rule::streamlet, input)
        .map_err(|e| {
            eprintln!("{}", e);
            TransformError::NoMatch
        })?
        .next()
        .unwrap();
    match pair.as_rule() {
        Rule::streamlet => Streamlet::try_from(pair),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logical::LogicalStreamType;
    use crate::streamlet::{Interface, Mode, StreamletBuilder};
    use crate::Name;
    use pest::Parser;
    use std::convert::TryInto;

    macro_rules! parse_ok {
        ($rule:ident, $string:literal) => {
            let r = SDFParser::parse(Rule::$rule, $string);
            match r {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
        };
    }

    #[test]
    fn test_ident() {
        parse_ok!(ident, "abc__123");
    }

    #[test]
    fn test_leaf_types() {
        parse_ok!(null, "Null");
        parse_ok!(bits, "Bits<10>");
    }

    #[test]
    fn test_field() {
        parse_ok!(field, "abc : Bits<1>");
        parse_ok!(field, "def1 : Null");
    }

    #[test]
    fn test_compound_types() {
        parse_ok!(group, "Group<a:Bits<1>, b:Bits<0>>");
        parse_ok!(union, "Union<a:Null, b:Bits<0>>");
    }

    #[test]
    fn test_complexity() {
        parse_ok!(compl, "4");
        parse_ok!(compl, "4.1");
        parse_ok!(compl, "4.1.2");
    }

    #[test]
    fn test_stream() {
        parse_ok!(stream, "Stream<Bits<1>>");
        parse_ok!(stream, "Stream<Bits<1>,t=0.5>");
        parse_ok!(stream, "Stream<Bits<1>,d=2>");
        parse_ok!(stream, "Stream<Bits<1>,c=4.0>");
        parse_ok!(stream, "Stream<Bits<1>,r=Reverse>");
        parse_ok!(stream, "Stream<Bits<1>,u=Group<u0:Bits<1>,u1:Bits<2>>>");
        parse_ok!(stream, "Stream<Bits<1>,x=false>");
        parse_ok!(
            stream,
            "Stream<Union<a: Null, b: Bits<1>, c: Group<d:Null, e:Null>>,t=0.5,d=2,c=4.0,r=Reverse,u=Group<u0:Bits<1>,u1:Bits<2>>,x=false>"
        );
    }

    #[test]
    fn test_mode() {
        parse_ok!(mode, "in");
        parse_ok!(mode, "out");
    }

    #[test]
    fn test_interface() {
        let _ = SDFParser::parse(Rule::interface, "some_name : in Stream<Bits<2>>;");
        parse_ok!(interface, "some_name : in Stream<Bits<2>>;");
    }
}
