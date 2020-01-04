use crate::ledger::errors::ProcessingError;
use crate::ledger::Entry;
use crate::ledger::Envelope;
use crate::ledger::envelope::EnvelopeType;
use crate::ledger::errors::ParseError;
use crate::ledger::utils;
use std::collections::HashMap;
use std::cmp::Ordering;

pub struct Account {
    name: String,
    expense_envelopes: HashMap<String, Envelope>,
    goal_envelopes: HashMap<String, Envelope>,
}

impl Account {
    pub fn parse(
        chunk: &str,
        decimal_symbol: char,
        date_format: &str,
    ) -> Result<Self, ParseError> {
        let mut lines = chunk.lines();
        let header = match lines.next() {
            Some(l) => l,
            None => {
                return Err(ParseError::new()
                    .set_context(chunk)
                    .set_message("account header can't be parsed because it doesn't exist"))
            }
        };

        let account_name = Account::parse_header(&header.to_string())?;
        let expense_envelopes = HashMap::new();
        let goal_envelopes = HashMap::new();

        let mut account = Account {
            name: account_name,
            expense_envelopes,
            goal_envelopes,
        };

        let mut envelope_chunk = String::new();
        for line in lines {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("expense") || trimmed_line.starts_with("goal") {
                // add a new envelope, if the chunk isn't blank
                if !envelope_chunk.trim().is_empty() {
                    let new_envelope =
                        Envelope::parse(&envelope_chunk, &account.name, decimal_symbol, &date_format)?;
                    account.add_envelope(new_envelope);
                }

                envelope_chunk = String::from(line);
            } else {
                envelope_chunk.push('\n');
                envelope_chunk.push_str(line);
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
            Ordering::Greater => {
                Err(ParseError {
                    context: Some(line.to_string()),
                    message: Some(
                        "accounts can't have spaces in them; use underscores instead: _".to_string(),
                    ),
                })
            },
            Ordering::Less => {
                Err(ParseError {
                    context: Some(line.to_string()),
                    message: Some("blank account definition".to_string()),
                })
            },
            Ordering::Equal => {
                Ok(tokens[1].to_string())
            }
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_envelope(&mut self, envelope: Envelope) {
        let envelope_collection = match envelope.get_type() {
            EnvelopeType::Expense => &mut self.expense_envelopes,
            EnvelopeType::Goal => &mut self.goal_envelopes,
        };

        match envelope_collection.get_mut(envelope.get_name()) {
            Some(e) => {
                e.merge(&envelope);
            }
            None => {
                envelope_collection.insert(envelope.get_name().to_string(), envelope);
            }
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
        let expense_len = self.expense_envelopes.len();
        let goal_len = self.goal_envelopes.len();

        // if no envelopes to display, quit
        if expense_len <= 0 && goal_len <= 0 {
            return
        }

        // displays account name at top
        println!("{}", self.name);

        // display expenses
        if expense_len > 0 {
            println!("  expenses");
            for (_, envelope) in self.expense_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        // display goals
        if goal_len > 0 {
            println!("  goals");
            for (_, envelope) in self.goal_envelopes.iter() {
                println!("{}", envelope);
            }
        }

        println!("");
    }
}
