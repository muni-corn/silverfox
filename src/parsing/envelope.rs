use std::str::FromStr;

use crate::{
    amount::Amount,
    envelope::{Frequency, FundingMethod, builder::EnvelopeBuilder, EnvelopeType},
    errors::{ParseError, SilverfoxError},
    parsing::{account_name, date},
};
use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::{alpha1, space0, space1, line_ending},
    combinator::{map, map_res, opt, value, recognize},
    multi::separated_list0,
    sequence::{pair, preceded, tuple},
    IResult,
};

use super::{amount, weekday, ordinal};

enum EnvelopeAttr {
    /// `amount <amount>` sets the funding goal for the envelope
    Amount(Amount),

    /// `<due|by|due by> <frequency>` sets when and how often the funding goal is due
    Due(Frequency),

    /// `for <account>` creates an auto-moving attribute
    For(String),

    /// `funding <fast|aggressive|slow|conservative>` sets the speed at which money moves.
    FundingMethod(FundingMethod),

    /// `starting <date>` sets the start date for funding an envelope
    Starting(NaiveDate),
}

/// Parses an envelope like this:
///
/// ```
/// expense rent due every 15th // due the 15th of every month
///     amount 1000             // for $1000
///     for expenses:home:rent  // automatically moves money when expenses:home:rent is used
///     funding aggressive      // use aggressive funding
/// ```
pub fn parse_envelope<'a>(
    parent_account_name: &'a str,
    date_format: &'a str,
    decimal_symbol: char,
) -> impl FnMut(&'a str) -> IResult<&'a str, EnvelopeBuilder, ParseError> {
    move |input| {
        let envelope_type_tag = alt((tag("envelope"), tag("goal"), tag("expense")));
        let (input, (_indent, _envelope_type)) = tuple((space1, envelope_type_tag))(input)?;
        let (input, _envelope_name) = preceded(space1, take_till1(char::is_whitespace))(input)?;

        let indent_separator = tuple((line_ending, tag(indent), space0));

        // get the remaining input and a Vec of EnvelopeAttrs
        let (_input, _attrs) = {
            let alt_parser = alt((
                amount_attr(decimal_symbol),
                due(date_format),
                for_clause,
                funding_method,
                starting(date_format),
            ));
            separated_list0(alt((recognize(space1), recognize(indent_separator))), alt_parser)(input)?
        };

        let builder = attrs.iter().fold(EnvelopeBuilder::new(envelope_name, EnvelopeType::from_str(envelope_type).unwrap(), parent_account_name), |acc, item| {
            match item {
                EnvelopeAttr::Amount(amount) => acc.amount(amount.clone()),
                EnvelopeAttr::Due(freq) => acc.freq(*freq),
                EnvelopeAttr::For(account) => acc.auto_account(account),
                EnvelopeAttr::FundingMethod(method) => acc.funding(*method),
                EnvelopeAttr::Starting(date) => acc.starting_date(*date),
            }
        });

        Ok((input, builder))
    }
}

/// Parses an `amount` field
fn amount_attr<'a>(
    decimal_symbol: char,
) -> impl FnMut(&'a str) -> IResult<&'a str, EnvelopeAttr, ParseError> {
    map(
        preceded(
            tuple((space0, tag("amount"), space1)),
            amount(decimal_symbol),
        ),
        EnvelopeAttr::Amount,
    )
}

/// Parses a `due` clause
fn due(date_format: &str) -> impl FnMut(&str) -> IResult<&str, EnvelopeAttr, ParseError> + '_ {
    move |input| {
        map(
            preceded(
                tuple((space0, alt((tag("due by"), tag("due"), tag("by"))), space1)),
                frequency(date_format),
            ),
            EnvelopeAttr::Due,
        )(input)
    }
}

/// Parses a `for` clause
fn for_clause(input: &str) -> IResult<&str, EnvelopeAttr, ParseError> {
    map(
        preceded(tuple((space0, tag("for"), space1)), account_name),
        |account_name| EnvelopeAttr::For(account_name.to_string()),
    )(input)
}

/// Parses a `funding` option
fn funding_method(input: &str) -> IResult<&str, EnvelopeAttr, ParseError> {
    map_res(
        preceded(tuple((space0, tag("funding"), space1)), alpha1),
        |method| match method {
            "fast" => Ok(EnvelopeAttr::FundingMethod(FundingMethod::Aggressive)),
            "slow" => Ok(EnvelopeAttr::FundingMethod(FundingMethod::Conservative)),
            _ => Err(SilverfoxError::Parse(ParseError {
                context: Some(input.to_string()),
                message: Some(format!(
                    "not a known funding method: {method}\n\ntry either `fast` or `slow`"
                )),
            })),
        },
    )(input)
}

/// Parses a `starting` clause
fn starting<'a>(
    date_format: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, EnvelopeAttr, ParseError> {
    map(
        preceded(tuple((space0, tag("starting"), space1)), date(date_format)),
        EnvelopeAttr::Starting,
    )
}

/// Whether a frequency is based on days, weeks, months, or years.
#[derive(Clone, Copy)]
enum FrequencyBase {
    /// The base is daily.
    Day,

    /// The base is weekly (on the weekday)
    Week(chrono::Weekday),

    /// The base is monthly (with day of month)
    Month(u32),

    /// The base is yearly (with date)
    Year(NaiveDate),
}

/// Parses a `Frequency` value
fn frequency(date_format: &str) -> impl FnMut(&str) -> IResult<&str, Frequency, ParseError> + '_ {
    move |input| {
        if let Ok((input, _)) = preceded::<_, _, _, ParseError, _, _>(space0, tag("every"))(input) {
            // preceded by `space0` is okay because we'll accept `everyother`
            if let Ok((input, _)) = preceded::<_, _, _, ParseError, _, _>(space0, tag("other"))(input) {
                map_res(frequency_base(date_format), |base| match base {
                    FrequencyBase::Day => Err(ParseError {
                        context: None,
                        message: Some("bidaily frequencies aren't supported yet".to_string()),
                    }),
                    FrequencyBase::Week(_) => Ok(Frequency::Biweekly),
                    FrequencyBase::Month(_) => Ok(Frequency::Bimonthly),
                    FrequencyBase::Year(_) => Err(ParseError {
                        context: None,
                        message: Some("biyearly due frequencies aren't supported yet".to_string()),
                    }),
                })(input)
            } else {
                map_res(frequency_base(date_format), |base| match base {
                    FrequencyBase::Day => Err(ParseError {
                        context: None,
                        message: Some("daily frequencies aren't supported yet".to_string()),
                    }),
                    FrequencyBase::Week(weekday) => Ok(Frequency::Weekly(weekday)),
                    FrequencyBase::Month(day) => Ok(Frequency::Monthly(day)),
                    FrequencyBase::Year(date) => Ok(Frequency::Annually(date)),
                })(input)
            }
        } else {
            map(
                preceded(
                    pair(space0, opt(pair(tag("once on"), space1))),
                    date(date_format),
                ),
                Frequency::Once,
            )(input)
        }
    }
}

/// Parses a `FrequencyBase`.
fn frequency_base(date_format: &str) -> impl FnMut(&str) -> IResult<&str, FrequencyBase, ParseError> + '_ {
    move |input| {
        alt((
            value(FrequencyBase::Day, tag("day")),
            map(weekday, FrequencyBase::Week),
            map(date(date_format), FrequencyBase::Year),
            map(ordinal, FrequencyBase::Month),
        ))(input)
    }
}
