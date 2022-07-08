use crate::{
    amount::AmountPool,
    entry::Entry,
    envelope::{Envelope, EnvelopeType},
    errors::*,
    posting::Posting,
    utils,
};
use std::cmp::Ordering;

mod builder;

pub struct Account {
    name: String,

    envelopes: Vec<Envelope>,

    /// The real, actual value of this account, which ignores envelopes or virtual postings.
    /// TODO: use this for balance statements
    real_value: AmountPool,
}

impl Account {
    pub fn parse(
        chunk: &str,
        decimal_symbol: char,
        date_format: &str,
    ) -> Result<Self, SilverfoxError> {
        let mut lines = chunk.lines();
        let header = match lines.next() {
            Some(l) => l,
            None => {
                return Err(SilverfoxError::from(ParseError {
                    context: Some(chunk.to_string()),
                    message: Some(
                        "account header can't be parsed because it doesn't exist".to_string(),
                    ),
                }))
            }
        };

        let account_name = Account::parse_header(&header.to_string())?;

        let mut account = Account {
            name: account_name,
            envelopes: Vec::new(),
            real_value: AmountPool::new(),
        };

        let mut envelope_chunk = String::new();
        for line in lines {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("expense") || trimmed_line.starts_with("goal") {
                // add a new envelope, if the chunk isn't blank
                if !envelope_chunk.trim().is_empty() {
                    let new_envelope = Envelope::parse(
                        &envelope_chunk,
                        &account.name,
                        decimal_symbol,
                        date_format,
                    )?;

                    if let Err(e) = account.add_envelope(new_envelope) {
                        return Err(SilverfoxError::from(e));
                    }
                }

                envelope_chunk = String::from(line);
            } else {
                envelope_chunk.push('\n');
                envelope_chunk.push_str(line);
            }
        }

        // parse the remainder
        if !envelope_chunk.trim().is_empty() {
            let new_envelope =
                Envelope::parse(&envelope_chunk, &account.name, decimal_symbol, date_format)?;

            if let Err(e) = account.add_envelope(new_envelope) {
                return Err(SilverfoxError::from(e));
            }
        }

        // finish by sorting envelopes

        Ok(account)
    }

    // returns the name of the account
    fn parse_header(mut line: &str) -> Result<String, ParseError> {
        // remove comments
        line = utils::remove_comments(line);

        let tokens = line.trim().split_whitespace().collect::<Vec<&str>>();
        match tokens.len().cmp(&2) {
            Ordering::Greater => Err(ParseError {
                context: Some(line.to_string()),
                message: Some(
                    "accounts can't have spaces in them; use underscores instead: _".to_string(),
                ),
            }),
            Ordering::Less => Err(ParseError {
                context: Some(line.to_string()),
                message: Some("blank account definition".to_string()),
            }),
            Ordering::Equal => Ok(tokens[1].to_string()),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_envelope(&mut self, envelope: Envelope) -> Result<(), ValidationError> {
        let envelope_exists = self
            .envelopes
            .iter()
            .any(|e| e.get_name() == envelope.get_name());

        if envelope_exists {
            Err(ValidationError {
                message: Some(format!(
                    "there's a duplicate envelope definition for `{}` in the account `{}`",
                    envelope.get_name(),
                    self.name
                )),
                context: None,
            })
        } else {
            self.envelopes.push(envelope);
            Ok(())
        }
    }

    /// Processes the Entry by looking for any changes to envelope amounts and applying them. Also
    /// adds to the real_value of the Account.
    pub fn process_entry(&mut self, entry: &Entry) -> Result<(), ProcessingError> {
        for envelope in self.envelopes.iter_mut() {
            envelope.process_entry(entry)?;
        }

        for posting in entry.get_postings() {
            if *posting.get_account() == self.name && !posting.is_envelope() {
                if let Some(a) = posting.get_amount() {
                    self.real_value += a;
                } else {
                    match entry.get_blank_amount() {
                        Ok(o) => {
                            if let Some(a) = o {
                                self.real_value += a;
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn display_envelopes(&self) {
        // if no envelopes to display, quit
        if self.envelopes.is_empty() {
            return;
        }

        // displays account name at top
        println!("{}", self.name);

        // display available balance
        println!("  available");
        let available_value = self.get_available_value();
        for amount in available_value.iter() {
            if amount.mag == 0.0 {
                continue;
            }
            println!("    {}", amount)
        }

        // display expenses
        let expense_envelopes: Vec<&Envelope> = self
            .envelopes
            .iter()
            .filter(|e| matches!(e.get_type(), EnvelopeType::Expense))
            .collect();
        if !expense_envelopes.is_empty() {
            println!("  expenses");
            for envelope in expense_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        // display goals
        let goal_envelopes: Vec<&Envelope> = self
            .envelopes
            .iter()
            .filter(|e| matches!(e.get_type(), EnvelopeType::Goal))
            .collect();
        if !goal_envelopes.is_empty() {
            println!("  goals");
            for envelope in goal_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        // display other envelopes
        let other_envelopes: Vec<&Envelope> = self
            .envelopes
            .iter()
            .filter(|e| matches!(e.get_type(), EnvelopeType::Generic))
            .collect();
        if !other_envelopes.is_empty() {
            println!("  other envelopes");
            for envelope in other_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        println!(); // do not remove; this is a separator
    }

    /// Sorts envelopes by due date and then returns postings that will fill (or drain) them as
    /// needed.
    pub fn get_filling_postings(&self) -> Vec<Posting> {
        // sort envelopes by due date (cloning so we don't have to mutate the Envelope)
        let mut sorted_envelopes = self.envelopes.clone();
        sorted_envelopes.sort_by_cached_key(|e| e.get_next_due_date());

        // generate and apply envelope-filling (or draining) postings from each envelope
        let (_final_available_value, postings) = self.get_available_value().iter().fold(
            (self.get_available_value(), Vec::new()),
            |(mut available_value, mut postings), available_amount| {
                // create a closure that can be used to create and apply postings for envelopes
                let apply_envelope_fill_posting = |envelope: &Envelope| {
                    // create a posting depending on what the envelope or account needs
                    let new_posting = Posting::from(envelope.get_filling_posting(&available_value));

                    // if the posting has an amount, subtract it (whether positive or negative)
                    // from the available value/pool that we're keeping track of and add the
                    // posting to the Vec of postings
                    if let Some(new_amount) = new_posting.get_amount() {
                        available_value -= new_amount;
                        postings.push(new_posting);
                    }
                };

                if available_amount.mag < 0. {
                    // if the available value in this amount's currency is below 0, we'll take money
                    // from the envelope whose due date is farthest away (by reversing the iterator)
                    sorted_envelopes
                        .iter()
                        .rev()
                        .for_each(apply_envelope_fill_posting);
                } else if available_amount.mag > 0. {
                    // otherwise, if the available value in this amount's currency is above 0,
                    // we'll fill envelopes in order of their next due dates
                    sorted_envelopes
                        .iter()
                        .for_each(apply_envelope_fill_posting);
                }

                (available_value, postings)
            },
        );

        postings
    }

    pub fn get_available_value(&self) -> AmountPool {
        let mut amount_pool = self.real_value.clone();
        for envelope in self.envelopes.iter() {
            amount_pool = amount_pool - envelope.get_next_amount() - envelope.get_now_amount();
        }

        amount_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Frequency;

    const ACCOUNT_STR: &str = "account assets:checking
             goal yearly_goal due every year starting 2020/2/20
                 amount 1000 CAD
             expense groceries due every 5th
                 amount 300 USD
                 for expenses:food:groceries
                 funding conservative";

    const BLANK_ACCOUNT_STR: &str = "account ";

    const ACCOUNT_WITH_SPACES_STR: &str = "account assets bank checking";

    const DEFAULT_DATE_FORMAT: &str = "%Y/%m/%d";

    #[test]
    fn test_parse() {
        // do the thing
        let account = match Account::parse(ACCOUNT_STR, '.', DEFAULT_DATE_FORMAT) {
            Ok(a) => a,
            Err(e) => panic!("{}", e),
        };

        // test name
        assert_eq!(account.name, "assets:checking");

        // test envelopes
        {
            // expenses
            let expense_envelopes: Vec<&Envelope> = account
                .envelopes
                .iter()
                .filter(|e| matches!(e.get_type(), EnvelopeType::Expense))
                .collect();
            assert_eq!(
                expense_envelopes.len(),
                1,
                "no expense envelopes; there should be one"
            );
            let ex_envelope = expense_envelopes.first().unwrap();
            assert_eq!(ex_envelope.get_name(), "groceries");
            assert_eq!(*ex_envelope.get_freq(), Frequency::Monthly(5));

            // goals
            let goal_envelopes: Vec<&Envelope> = account
                .envelopes
                .iter()
                .filter(|e| matches!(e.get_type(), EnvelopeType::Goal))
                .collect();
            assert_eq!(
                goal_envelopes.len(),
                1,
                "no goal envelopes; there should be one"
            );
            let goal_envelope = goal_envelopes.first().unwrap();
            assert_eq!(goal_envelope.get_name(), "yearly_goal");
            assert_eq!(
                *goal_envelope.get_freq(),
                Frequency::Annually(chrono::NaiveDate::from_ymd(2020, 2, 20))
            );
        }
    }

    #[test]
    fn blank_account_test() {
        let result = Account::parse(BLANK_ACCOUNT_STR, '.', DEFAULT_DATE_FORMAT);
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_spaces_test() {
        let result = Account::parse(ACCOUNT_WITH_SPACES_STR, '.', DEFAULT_DATE_FORMAT);
        assert!(result.is_err());
    }
}
