use std::str::FromStr;

use crate::errors::ParseError;
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_while, take_while1},
    character::complete::space0,
    combinator::{map, map_res},
    sequence::preceded,
    sequence::{pair, terminated},
    IResult,
};

pub fn eol_comment(line: &str) -> IResult<&str, &str, ParseError> {
    let comment_start = alt((tag("//"), tag(";")));
    map(
        pair(comment_start, preceded(space0, is_not("\r\n"))),
        |(_, y)| y,
    )(line)
}

/// Returns true if the char is a digit, period, comma, underscore, or dash. Either a period,
/// comma, or underscore can be used as a thousands separator.
pub fn is_amount_quantity_char(c: char) -> bool {
    c.is_digit(10) || "-,._".contains(c)
}

/// Returns true if the char can qualify as part of a symbol for an amount.
pub fn is_amount_symbol_char(c: char) -> bool {
    !is_amount_quantity_char(c) && !c.is_whitespace() && !";/@=!".contains(c)
}

/// Parses and returns a date provided a custom format
pub fn date<'a>(format: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, NaiveDate, ParseError> {
    move |input: &str| {
        if format.chars().any(|c| c.is_whitespace()) {
            Err(nom::Err::Failure(ParseError {
                context: Some(format.to_string()),
                message: Some(String::from("your date format cannot contain spaces")),
            }))
        } else {
            map_res(take_till1(char::is_whitespace), |s| NaiveDate::parse_from_str(s, format))(input)
        }
    }
}

pub fn account_name<'a>(input: &'a str) -> IResult<&'a str, &'a str, ParseError> {
    is_not(" \t\n\r")(input)
}

mod tests {
    use super::*;

    #[test]
    fn test_eol_comment() {
        assert_eq!(
            eol_comment("// this is a slash comment"),
            Ok(("", "this is a slash comment"))
        );
        assert_eq!(
            eol_comment("; this is a semicolon comment"),
            Ok(("", "this is a semicolon comment"))
        );

        // we've opted to preserve extra comment symbols
        assert_eq!(
            eol_comment("//// thicc comment"),
            Ok(("", "// thicc comment"))
        );
        assert_eq!(
            eol_comment(";;;; also thicc comment"),
            Ok(("", ";;; also thicc comment"))
        );

        assert_eq!(
            eol_comment("//nice and comfortable"),
            Ok(("", "nice and comfortable"))
        );
        assert_eq!(eol_comment(";cozy"), Ok(("", "cozy")));

        assert_eq!(eol_comment("///thicc"), Ok(("", "/thicc")));
        assert_eq!(eol_comment(";;;boi"), Ok(("", ";;boi")));

        assert!(eol_comment(" // won't parse if spaces come before").is_err());
        assert!(eol_comment("/ two slashes are needed").is_err());
        assert!(eol_comment("\t; tabs aren't allowed at the beginning either").is_err());
    }
}
