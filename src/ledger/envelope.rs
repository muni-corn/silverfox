use crate::ledger::errors::ParseError;
use crate::ledger::utils;
use crate::ledger::Amount;
use std::collections::HashSet;
use std::str::FromStr;
use std::cmp::Ordering;

pub struct Envelope {
    name: String,
    amount: Amount,
    envelope_type: EnvelopeType,
    for_accounts: HashSet<String>,
    freq: Frequency,
    funding: FundingMethod,
}

pub enum EnvelopeType {
    Expense,
    Goal,
}

impl EnvelopeType {
    fn from_str(raw: &str) -> Result<Self, ParseError> {
        match raw {
            "expense" => Ok(EnvelopeType::Expense),
            "goal" => Ok(EnvelopeType::Goal),
            _ => {
                Err(ParseError {
                    context: Some(raw.to_string()),
                    message: Some("this envelope type doesn't exist; instead use either `expense` or `goal`".to_string())
                })
            }
        }
    }
}

pub enum FundingMethod {
    Manual,
    Conservative,
    Aggressive,
}

impl FundingMethod {
    fn from_str(raw: &str) -> Result<Self, ParseError> {
        match raw {
            "manual" => Ok(FundingMethod::Manual),
            "aggressive" => Ok(FundingMethod::Aggressive),
            "conservative" => Ok(FundingMethod::Conservative),
            _ => {
                Err(ParseError {
                    context: Some(raw.to_string()),
                    message: Some("this funding method doesn't exist".to_string()),
                })
            }
        }
    }
}

// tuples including a date is the "starting" date
pub enum Frequency {
    Never,
    Once(chrono::NaiveDate),
    Weekly(Option<chrono::NaiveDate>, chrono::Weekday),
    Biweekly(chrono::NaiveDate, chrono::Weekday),
    Monthly(Option<chrono::NaiveDate>, i8),
    Bimonthly(chrono::NaiveDate, i8),
    // Quarterly(chrono::NaiveDate),
    // Semiannually(chrono::NaiveDate),
    // Annually(chrono::NaiveDate),
}

impl Frequency {
    pub fn parse(mut s: &str, date_format: &str) -> Result<Self, ParseError> {
        // parse `starting` clause
        let (starting_date, starting_idx) = match Self::extract_starting(s, date_format) {
            Ok(o) => match o {
                Some(t) => (Some(t.0), Some(t.1)),
                None => (None, None)
            },
            Err(e) => return Err(e)
        };

        // if starting exists, trim the string supplied so that `starting` is cut off
        if let Some(i) = starting_idx {
            s = &s[..i];
        }

        if s.starts_with("every other ") {
            // stop if "starting" isn't given, since it's required here
            if starting_date.is_none() {
                return Err(ParseError {
                    context: Some(s.to_string()),
                    message: Some("a `starting` clause is required for `every other` frequencies so mvelopes knows which weeks or months to use".to_string())
                })
            }

            // parse "every others"
            // remember: the `starting` clause is already trimmed
            let what = &s["every other ".len()..];

            if let Some(w) = Self::parse_weekday(what) {
                Ok(Self::Biweekly(starting_date.unwrap(), w))
            } else if let Some(d) = Self::parse_day_of_month(what) {
                Ok(Self::Bimonthly(starting_date.unwrap(), d))
            } else {
                Err(ParseError {
                    context: Some(s.to_string()),
                    message: Some("invalid frequency".to_string())
                })
            }
        } else if s.starts_with("every ") {
            // parse "everys"
            // remember: the `starting` clause is already trimmed
            let what = &s["every ".len()..];

            if let Some(w) = Self::parse_weekday(what) {
                Ok(Self::Weekly(starting_date, w))
            } else if let Some(d) = Self::parse_day_of_month(what) {
                Ok(Self::Monthly(starting_date, d))
            } else {
                Err(ParseError {
                    context: Some(s.to_string()),
                    message: Some("invalid frequency".to_string())
                })
            }
        } else {
            // probably not repeating, so let's just try to parse a date
            match chrono::NaiveDate::parse_from_str(s, date_format) {
                Ok(d) => {
                    Ok(Self::Once(d))
                },
                Err(_) => {
                    let message = format!("couldn't parse `{}` with format `{}`", s, date_format);

                    Err(ParseError {
                        message: Some(message),
                        context: None
                    })
                }
            }
        }
    }

    /// Extracts and parses the `starting` clause of an Envelope. The Result returned uses an
    /// Option because a `starting` may exist or not exist.
    fn extract_starting(s: &str, date_format: &str) -> Result<Option<(chrono::NaiveDate, usize)>, ParseError> {
        let idx = match s.find(" starting ") {
            Some(i) => i,
            None => return Ok(None)
        };

        match chrono::NaiveDate::parse_from_str(&s[idx..], date_format) {
            Ok(d) => {
                let result = Some((d, idx));
                Ok(result)
            },
            Err(_) => {
                let message = format!("couldn't parse starting date `{}` with format `{}`", s, date_format);

                Err(ParseError {
                    message: Some(message),
                    context: Some(s.to_string())
                })
            }
        }
    }

    /// Parses, you know, a Weekday. Returns an Option because it may or may not exist.
    fn parse_weekday(s: &str) -> Option<chrono::Weekday> {
        match chrono::Weekday::from_str(s) {
            Ok(w) => Some(w),
            _ => None
        }
    }

    /// Parses a day of the month. Returns an Option because it may or may not exist. Any
    /// non-digits are removed from the string to parse a number. So, technically, you could write
    /// "1stjalsdkxbuz" and it would still return 1. "2faxcbya7uw" would return 27.
    fn parse_day_of_month(s: &str) -> Option<i8> {
        // filter out any letters, spaces, etc, and parse the number in the string
        let num = s.chars().filter(|c| c.is_digit(10)).collect::<String>();

        match num.parse::<i8>() {
            Ok(n) => Some(n),
            _ => None
        }
    }
}

impl Envelope {
    pub fn parse(chunk: &str, account_name: &str, decimal_symbol: char, date_format: &str) -> Result<Self, ParseError> {
        let mut lines = chunk.lines();

        let mut envelope = if let Some(l) = lines.nth(0) {
            Self::from_header(l, date_format)?
        } else {
            let err = ParseError::new().set_context(&chunk).set_message("envelope header can't be parsed because it doesn't exist");
            return Err(err)
        };

        // parse the body
        let body_vec = lines.skip(1).collect::<Vec<&str>>();
        let body = body_vec.join(" ");
        envelope.add_body(&body, account_name, decimal_symbol)?;

        Ok(envelope)
    }

    /// Returns the starting struct of an Envelope. The string passed in can include ledger
    /// comments.
    fn from_header(header: &str, date_format: &str) -> Result<Self, ParseError> {
        let tokens = utils::remove_comments(header)
            .trim()
            .split_whitespace()
            .collect::<Vec<&str>>();

        if tokens.len() < 2 {
            return Err(ParseError {
                context: Some(header.to_string()),
                message: Some("blank account definition".to_string())
            })
        }

        let envelope_type = match EnvelopeType::from_str(&tokens[0]) {
            Ok(t) => t,
            Err(e) => return Err(e)
        };

        let freq = match Self::extract_frequency(header, date_format) {
            Ok(f) => f,
            Err(e) => return Err(e)
        };

        let envelope = Envelope {
            name: String::from(tokens[1]),
            amount: Amount::zero(),
            funding: FundingMethod::Manual,
            envelope_type,
            freq,
            for_accounts: HashSet::new(),
        };
        Ok(envelope)
    }

    fn add_body(&mut self, body: &str, account_name: &str, decimal_symbol: char) -> Result<(), ParseError> {
        for line in body.lines() {
            let trimmed_line = utils::remove_comments(line).trim();
            let line_split = trimmed_line.split_whitespace().collect::<Vec<&str>>();
            let key = line_split[0];

            // get the index of the first space (because that's where the values begins)
            let idx;
            match trimmed_line.find(' ') {
                Some(i) => idx = i,
                None => {
                    let message = format!("the property `{}` to an envelope (`{}` in {}) is blank", line_split[0], self.name, account_name);
                    let err = ParseError::new().set_message(&message);
                    return Err(err)
                }
            }

            let value = &trimmed_line[idx..].to_string();

            if line_split.len() > 1 {
                match key {
                    "for" => {
                        // parse a `for` property, which should only include an account (no spaces,
                        // of course)
                        if let Err(e) = self.add_account(value) {
                            return Err(e)
                        }
                    }
                    "amount" => {
                        // set the amount of the envelope
                        match Amount::parse(value, decimal_symbol) {
                            Ok(a) => self.amount = a,
                            Err(e) => return Err(e)
                        }
                    }
                    "funding" => {
                        // parse the funding method for the envelope
                        match FundingMethod::from_str(value) {
                            Ok(f) => self.funding = f,
                            Err(e) => return Err(e)
                        }
                    },
                    _ => {
                        return Err(ParseError::new().set_message(format!("the `{}` property isn't understood by mvelopes", key).as_str()))
                    }
                }
            }
        }

        Ok(())
    }

    fn add_account(&mut self, s: &str) -> Result<(), ParseError> {
        let arg_count = s.split_whitespace().count();

        match arg_count.cmp(&1) {
            Ordering::Greater => {
                // more than one token? account probably has spaces in it
                Err(ParseError {
                    message: Some("remember that account names can't contain spaces; this `for` property couldn't be parsed correctly".to_string()),
                    context: Some(s.to_string())
                })
            },
            Ordering::Less => {
                // something less than one token? that's an issue
                Err(ParseError {
                    message: Some("a `for` property is blank".to_string()),
                    context: Some(s.to_string()),
                })
            },
            Ordering::Equal => {
                // exactly one token? perfect
                self.for_accounts.insert(String::from(s));
                Ok(())
            }
        }
    }

    fn extract_frequency(header: &str, date_format: &str) -> Result<Frequency, ParseError> {
        let clean_header = utils::remove_comments(header);
        if clean_header.contains("no date") {
            return Ok(Frequency::Never)
        }

        let frequency_index; 
        // first try matching by "by", since it will always come after "by" whether "by" or "due by"
        match clean_header.rfind("by ") {
            Some(i) => {
                frequency_index = i + "by ".len();
            },
            // if not found, search for "due"
            None => match clean_header.rfind("due ") {
                Some(i) => {
                    frequency_index = i + "due ".len();
                },
                // if that's not found, then pbpbpbpbpbpbpbpbpbp
                None => return Err(ParseError {
                    message: Some("couldn't figure out when this envelope is due; use `no date` if you don't want to specify a due date".to_string()),
                    context: Some(clean_header.to_string())
                })
            }
        }

        let raw_freq = &clean_header[frequency_index..];
        Frequency::parse(raw_freq, date_format)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn merge(&mut self, other: &Envelope) {}
}
