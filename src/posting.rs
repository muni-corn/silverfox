use crate::amount::Amount;
use crate::errors::*;
use crate::utils;
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Posting {
    // NOTE: as an idea, potentially make Posting an enum in the future, alongside structs
    // NormalPosting and EnvelopePosting. the enum values are Normal and Envelope, respectively.
    // the two types of postings should really be separated

    amount: Option<Amount>,
    account: String,
    price_assertion: Option<Amount>,
    balance_assertion: Option<Amount>,
    total_balance_assertion: Option<Amount>,

    /// Provided as Some if this posting is an explicit envelope posting
    envelope_name: Option<String>,
}

impl Posting {
    pub fn new(
        account: &str,
        envelope_name: Option<String>,
        amount: Option<Amount>,
        price_assertion: Option<Amount>,
        balance_assertion: Option<Amount>,
        total_balance_assertion: Option<Amount>,
    ) -> Self {
        Self {
            account: String::from(account),
            amount,
            envelope_name,
            balance_assertion,
            total_balance_assertion,
            price_assertion,
        }
    }

    pub fn new_envelope_posting(account: String, amount: Amount, envelope_name: String) -> Self {
        Self {
            account,
            amount: Some(amount),
            envelope_name: Some(envelope_name),
            balance_assertion: None,
            total_balance_assertion: None,
            price_assertion: None,
        }
    }

    fn blank() -> Self {
        Self {
            amount: None,
            account: String::new(),
            price_assertion: None,
            balance_assertion: None,
            total_balance_assertion: None,
            envelope_name: None,
        }
    }

    pub fn parse(
        line: &str,
        decimal_symbol: char,
        accounts: &HashSet<&String>,
    ) -> Result<Self, MvelopesError> {
        let mut posting = Self::blank();

        // remove comments and other impurities
        let trimmed_line = utils::remove_comments(line).trim();
        let tokens = trimmed_line.split_whitespace().collect::<Vec<&str>>();

        let amount_tokens: Vec<&str>;
        match tokens[0] {
            "envelope" => {
                posting.account = tokens[1].to_string();
                posting.envelope_name = Some(tokens[2].to_string());
                amount_tokens = tokens[3..].to_vec();
            }
            _ => {
                posting.account = tokens[0].to_string();
                amount_tokens = tokens[1..].to_vec();
            }
        }

        if let Err(e) = posting.parse_amount(&amount_tokens, decimal_symbol) {
            Err(MvelopesError::from(e))
        } else if let Err(e) = posting.parse_assertion_amounts(&amount_tokens, decimal_symbol) {
            Err(MvelopesError::from(e))
        } else if let Err(e) = posting.validate(&accounts) {
            Err(MvelopesError::from(e))
        } else {
            Ok(posting)
        }
    }

    pub fn get_native_value(&self) -> Option<f64> {
        // calculate native price of this posting. posting.amount must exist for this to work
        // (since this is literally used primarily for calculating the value of blank posting
        // amounts, boi)
        if let Some(a) = &self.amount {
            if a.symbol.is_none() {
                // if the posting's amount is native, then of course that's the native amount
                Some(a.mag)
            } else {
                match &self.price_assertion {
                    Some(p) => {
                        if p.symbol.is_none() {
                            // otherwise, if the price assertion is a native amount, we'll use that to
                            // determine the native value
                            Some(a.mag * p.mag)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            }
        } else {
            None
        }
    }

    fn parse_amount(
        &mut self,
        amount_tokens: &[&str],
        decimal_symbol: char,
    ) -> Result<(), ParseError> {
        let mut iter = amount_tokens.iter();
        let raw_amount = match iter.position(|&s| s == "@" || s == "!" || s == "=" || s == "!!") {
            Some(cutoff) => amount_tokens[..cutoff].join(" "),
            None => amount_tokens.join(" "),
        };

        if raw_amount.trim().is_empty() {
            self.amount = None;
            return Ok(());
        }

        self.amount = match Amount::parse(&raw_amount, decimal_symbol) {
            Ok(a) => Some(a),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn parse_assertion_amounts(
        &mut self,
        amount_tokens: &[&str],
        decimal_symbol: char,
    ) -> Result<(), ParseError> {
        self.balance_assertion =
            match Self::parse_balance_assertion_amount(amount_tokens, decimal_symbol) {
                Ok(a) => a,
                Err(e) => return Err(e),
            };

        self.total_balance_assertion =
            match Self::parse_total_balance_assertion_amount(amount_tokens, decimal_symbol) {
                Ok(a) => a,
                Err(e) => return Err(e),
            };

        self.price_assertion = match Self::parse_price_amount(amount_tokens, decimal_symbol) {
            Ok(price_opt) => {
                // parsing succeeded, if there is a price, use that
                if let Some(price) = price_opt {
                    Some(price)
                } else {
                    // if no price, try to parse total cost
                    match Self::parse_total_cost_amount(amount_tokens, decimal_symbol) {
                        Ok(total_cost_opt) => {
                            // successful, so see if something's there
                            if let Some(total_cost) = total_cost_opt {
                                match &self.amount {
                                    Some(a) => {
                                        // to determine price, we have to figure it out by dividing
                                        // the total cost by the original amount of this posting

                                        let calculated_price_amt = Amount {
                                            mag: total_cost.mag / a.mag,
                                            symbol: total_cost.symbol
                                        };

                                        Some(calculated_price_amt)
                                    },
                                    None => return Err(ParseError::default().set_message("a total cost assertion can't be supplied if the posting has no amount"))
                                }
                            } else {
                                // nothing there? nothing will be used
                                None
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Err(e) => return Err(e),
        };

        Ok(())
    }

    fn parse_balance_assertion_amount(
        amount_tokens: &[&str],
        decimal_symbol: char,
    ) -> Result<Option<Amount>, ParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "!", |&s| {
            s == "!!" || s == "@" || s == "="
        })
    }

    fn parse_total_balance_assertion_amount(
        amount_tokens: &[&str],
        decimal_symbol: char,
    ) -> Result<Option<Amount>, ParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "!!", |&s| {
            s == "!" || s == "@" || s == "="
        })
    }

    fn parse_price_amount(
        amount_tokens: &[&str],
        decimal_symbol: char,
    ) -> Result<Option<Amount>, ParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "@", |&s| {
            s == "!" || s == "!!" || s == "="
        })
    }

    fn parse_total_cost_amount(
        amount_tokens: &[&str],
        decimal_symbol: char,
    ) -> Result<Option<Amount>, ParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "=", |&s| {
            s == "!" || s == "!!" || s == "@"
        })
    }

    fn extract_amount<P>(
        amount_tokens: &[&str],
        decimal_symbol: char,
        wanted_operator: &str,
        unwanted_op_predicate: P,
    ) -> Result<Option<Amount>, ParseError>
    where
        P: FnMut(&&str) -> bool,
    {
        // find the balance_assertion token
        let mut iter = amount_tokens.iter();
        match iter.position(|&s| s == wanted_operator) {
            Some(i) => {
                // trim unwanted tokens
                let mut useful_tokens = &amount_tokens[i + 1..];

                // find any other tokens that should be filtered out
                if let Some(i) = useful_tokens.iter().position(unwanted_op_predicate) {
                    // trim unwanted tokens
                    useful_tokens = &useful_tokens[..i];
                }

                // parse the amount
                match Amount::parse(useful_tokens.join(" ").as_str(), decimal_symbol) {
                    Ok(a) => Ok(Some(a)),
                    Err(e) => Err(e),
                }
            }
            None => Ok(None),
        }
    }

    fn validate(&self, accounts: &HashSet<&String>) -> Result<(), ValidationError> {
        if !accounts.contains(&self.account) {
            let message = format!(
                "the account `{}` is not defined in your journal",
                self.account
            );
            Err(ValidationError::default().set_message(message.as_str()))
        } else {
            Ok(())
        }
    }

    // getters

    /// Returns the Posting's Amount
    pub fn get_amount(&self) -> &Option<Amount> {
        &self.amount
    }

    /// Returns the Posting's account
    pub fn get_account(&self) -> &String {
        &self.account
    }

    pub fn get_envelope_name(&self) -> Option<&String> {
        self.envelope_name.as_ref()
    }

    pub fn as_parsable(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for Posting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut postlude = String::new();

        if let Some(a) = &self.amount {
            postlude.push_str(&a.display());
        }

        if let Some(p) = &self.price_assertion {
            postlude.push_str(&format!(" @ {}", p));
        }

        if let Some(b) = &self.balance_assertion {
            postlude.push_str(&format!(" ! {}", b));
        }

        if let Some(t) = &self.total_balance_assertion {
            postlude.push_str(&format!(" !! {}", t));
        }

        if let Some(n) = &self.envelope_name {
            let prelude = format!("envelope {} {}", self.account, n);
            write!(f, "{:50} {}", prelude, postlude)
        } else {
            write!(f, "{:50} {}", self.account, postlude)
        }
    }
}
