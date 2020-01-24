use crate::{
    parser::{nonempty_comma_list, r#type, space_opt, usize},
    Data,
};
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::map,
    sequence::separated_pair, IResult,
};

/// Parses an Empty.
pub fn empty(input: &str) -> IResult<&str, Data> {
    map(tag("Empty"), |_| Data::Empty)(input)
}

/// Parses a Prim<B>.
pub fn prim(input: &str) -> IResult<&str, Data> {
    map(r#type("Prim", usize), Data::Prim)(input)
}

/// Parses a Struct<T, U, ...>.
pub fn r#struct(input: &str) -> IResult<&str, Data> {
    map(
        r#type("Struct", nonempty_comma_list(data_type)),
        Data::Struct,
    )(input)
}

/// Parses a Tuple<T, n>.
pub fn tuple(input: &str) -> IResult<&str, Data> {
    map(
        r#type(
            "Tuple",
            separated_pair(data_type, space_opt(char(',')), usize),
        ),
        |(data_type, count)| Data::Tuple(Box::new(data_type), count),
    )(input)
}

/// Parses a Seq<T>.
pub fn seq(input: &str) -> IResult<&str, Data> {
    map(r#type("Seq", data_type), |data_type| {
        Data::Seq(Box::new(data_type))
    })(input)
}

/// Parses a Variant<T, U, ...>.
pub fn variant(input: &str) -> IResult<&str, Data> {
    map(
        r#type("Variant", nonempty_comma_list(data_type)),
        Data::Variant,
    )(input)
}

/// Parses a Data type.
pub fn data_type(input: &str) -> IResult<&str, Data> {
    alt((variant, seq, tuple, r#struct, prim, empty))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        assert_eq!(empty("Empty"), Ok(("", Data::Empty)));
    }

    #[test]
    fn parse_prim() {
        assert_eq!(prim("Prim<8>"), Ok(("", Data::Prim(8))));
        assert!(prim("Prim<>").is_err());
        assert!(prim("prim<8>").is_err());
    }

    #[test]
    fn parse_struct() {
        assert_eq!(
            r#struct("Struct<Prim<3>>"),
            Ok(("", Data::Struct(vec![Data::Prim(3)])))
        );
        assert_eq!(
            r#struct("Struct<Struct<Prim<3>>>"),
            Ok(("", Data::Struct(vec![Data::Struct(vec![Data::Prim(3)])])))
        );
        assert_eq!(
            r#struct("Struct<Struct<Prim<3>,Prim<8>>>"),
            Ok((
                "",
                Data::Struct(vec![Data::Struct(vec![Data::Prim(3), Data::Prim(8)])])
            ))
        );
        assert_eq!(
            r#struct("Struct<Struct<Prim<3>,Prim<8>>>"),
            r#struct("Struct<Struct<Prim<3>, Prim<8>>>"),
        );
    }

    #[test]
    fn parse_tuple() {
        assert_eq!(
            tuple("Tuple<Prim<8>,4>"),
            Ok(("", Data::Tuple(Box::new(Data::Prim(8)), 4)))
        );
    }

    #[test]
    fn parse_seq() {
        assert_eq!(
            seq("Seq<Tuple<Prim<8>,4>>"),
            Ok((
                "",
                Data::Seq(Box::new(Data::Tuple(Box::new(Data::Prim(8)), 4)))
            ))
        );
    }

    #[test]
    fn parse_variant() {
        assert_eq!(
            variant("Variant<Prim<8>, Seq<Tuple<Prim<8>,4>>>"),
            Ok((
                "",
                Data::Variant(vec![
                    Data::Prim(8),
                    Data::Seq(Box::new(Data::Tuple(Box::new(Data::Prim(8)), 4)))
                ])
            ))
        );
    }

    #[test]
    fn parse_data_type() {
        assert_eq!(data_type("Prim<8>"), Ok(("", Data::Prim(8))));
        assert_eq!(
            data_type("Struct<Prim<4>, Prim<4>>"),
            Ok(("", Data::Struct(vec![Data::Prim(4), Data::Prim(4)])))
        );
        assert_eq!(
            data_type("Tuple<Prim<8>, 4>"),
            Ok(("", Data::Tuple(Box::new(Data::Prim(8)), 4)))
        );
        assert_eq!(
            data_type("Seq<Prim<4>>"),
            Ok(("", Data::Seq(Box::new(Data::Prim(4)))))
        );
        assert_eq!(
            data_type("Variant<Prim<4>,Prim<4>>"),
            Ok(("", Data::Variant(vec![Data::Prim(4), Data::Prim(4)])))
        );
    }
}
