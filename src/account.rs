use crate::amount::AmountPool;
use crate::entry::Entry;
use crate::envelope::{Envelope, EnvelopeType};
use crate::errors::*;
use crate::posting::Posting;
use crate::utils;
use std::cmp::Ordering;
use std::collections::HashMap;

pub struct Account {
    name: String,
    expense_envelopes: HashMap<String, Envelope>,
    goal_envelopes: HashMap<String, Envelope>,

    /// The real, actual value of this account, which ignores envelopes or virtual postings.
    /// TODO: use this for balance statements
    real_value: AmountPool,
}

impl Account {
    pub fn parse(
        chunk: &str,
        decimal_symbol: char,
        date_format: &str,
    ) -> Result<Self, MvelopesError> {
        let mut lines = chunk.lines();
        let header = match lines.next() {
            Some(l) => l,
            None => {
                return Err(MvelopesError::from(
                    ParseError::default()
                        .set_context(chunk)
                        .set_message("account header can't be parsed because it doesn't exist"),
                ))
            }
        };

        let account_name = Account::parse_header(&header.to_string())?;
        let expense_envelopes = HashMap::new();
        let goal_envelopes = HashMap::new();

        let mut account = Account {
            name: account_name,
            expense_envelopes,
            goal_envelopes,
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
                        &date_format,
                    )?;

                    if let Err(e) = account.add_envelope(new_envelope) {
                        return Err(MvelopesError::from(e));
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
            let new_envelope = Envelope::parse(
                &envelope_chunk,
                &account.name,
                decimal_symbol,
                &date_format,
            )?;

            if let Err(e) = account.add_envelope(new_envelope) {
                return Err(MvelopesError::from(e));
            }
        }


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
        let envelope_collection = match envelope.get_type() {
            EnvelopeType::Expense => &mut self.expense_envelopes,
            EnvelopeType::Goal => &mut self.goal_envelopes,
        };

        if envelope_collection.get_mut(envelope.get_name()).is_some() {
            Err(ValidationError {
                message: Some(format!(
                                 "there's a duplicate envelope definition for `{}` in the account `{}`",
                                 envelope.get_name(),
                                 self.name
                         )),
                         context: None,
            })
        } else {
            (*envelope_collection).insert(envelope.get_name().to_string(), envelope);
            Ok(())
        }
    }

    /// Processes the Entry by looking for any changes to envelope amounts and applying them
    pub fn process_entry(&mut self, entry: &Entry) -> Result<(), ProcessingError> {
        for (_, envelope) in self.expense_envelopes.iter_mut() {
            envelope.process_entry(entry)?;
        }

        for (_, envelope) in self.goal_envelopes.iter_mut() {
            envelope.process_entry(entry)?;
        }

        Ok(())
    }

    pub fn display_envelopes(&self) {
        // if no envelopes to display, quit
        if self.expense_envelopes.is_empty() && self.goal_envelopes.is_empty() {
            return;
        }

        // displays account name at top
        println!("{}", self.name);

        // display expenses
        if !self.expense_envelopes.is_empty() {
            println!("  expenses");
            for (_, envelope) in self.expense_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        // display goals
        if !self.goal_envelopes.is_empty() {
            println!("  goals");
            for (_, envelope) in self.goal_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        println!(); // do not remove; this is a separator
    }

    pub fn get_filling_postings(&self) -> Vec<Posting> {
        let mut postings: Vec<Posting> = Vec::new();
        let available_value = self.get_available_value();

        for (_, envelope) in self.expense_envelopes.iter() {
            postings.push(envelope.get_filling_posting(&available_value));
        }

        for (_, envelope) in self.goal_envelopes.iter() {
            postings.push(envelope.get_filling_posting(&available_value));
        }

        postings
    }

    pub fn get_available_value(&self) -> AmountPool {
        let mut amount_pool = self.real_value.clone();
        for (_, envelope) in self.expense_envelopes.iter() {
            amount_pool = amount_pool - envelope.get_next_amount() - envelope.get_now_amount();
        }

        for (_, envelope) in self.goal_envelopes.iter() {
            amount_pool = amount_pool - envelope.get_next_amount() - envelope.get_now_amount();
        }

        amount_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Frequency;

    const ACCOUNT_STR: &'static str =
        "account assets:checking
             goal yearly_goal due every year starting 2020/2/20
                 amount 1000 CAD
             expense groceries due every 5th
                 amount 300 USD
                 for expenses:food:groceries
                 funding conservative";

    #[test]
    fn test_parse() {
        // do the thing
        let account = match Account::parse(ACCOUNT_STR, '.', "%Y/%m/%d") {
            Ok(a) => a,
            Err(e) => panic!(e)
        };

        // test name
        assert_eq!(account.name, "assets:checking");

        // test envelopes
        {
            // expenses
            assert_eq!(account.expense_envelopes.len(), 1, "no expense envelopes; there should be one");
            let ex_envelope = &account.expense_envelopes["groceries"];
            assert_eq!(ex_envelope.get_name(), "groceries");
            assert_eq!(*ex_envelope.get_freq(), Frequency::Monthly(5));

            // goals
            assert_eq!(account.goal_envelopes.len(), 1, "no goal envelopes; there should be one");
            let goal_envelope = &account.goal_envelopes["yearly_goal"];
            assert_eq!(goal_envelope.get_name(), "yearly_goal");
            assert_eq!(*goal_envelope.get_freq(), Frequency::Annually(chrono::NaiveDate::from_ymd(2020, 2, 20)));
        }
    }
}
