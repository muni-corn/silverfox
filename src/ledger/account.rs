use crate::ledger::envelope::Envelope;
use crate::ledger::errors::ChunkParseError;
use crate::ledger::utils;
use std::collections::HashMap;
use std::cmp::Ordering;

pub struct Account {
    name: String,
    envelopes: HashMap<String, Envelope>,
}

impl Account {
    pub fn parse(
        chunk: &str,
        decimal_symbol: char,
        date_format: &str,
    ) -> Result<Self, ChunkParseError> {
        let mut lines = chunk.lines();
        let header = match lines.nth(0) {
            Some(l) => l,
            None => {
                return Err(ChunkParseError::new()
                    .set_chunk(chunk)
                    .set_message("account header can't be parsed because it doesn't exist"))
            }
        };

        let account_name = Account::parse_header(&header.to_string())?;
        let envelopes: HashMap<String, Envelope> = HashMap::new();

        let mut account = Account {
            name: account_name,
            envelopes,
        };

        let mut envelope_chunk = String::new();
        for line in lines.skip(1) {
            let trimmed_line = line.trim();
            if trimmed_line.starts_with("expense") || trimmed_line.starts_with("goal") {
                // add a new envelope
                let new_envelope =
                    Envelope::parse(&envelope_chunk, &account.name, decimal_symbol, &date_format)?;
                account.add_envelope(new_envelope);

                envelope_chunk = String::from(line);
            } else {
                envelope_chunk.push('\n');
                envelope_chunk.push_str(line);
            }
        }

        Ok(account)
    }

    // returns the name of the account
    fn parse_header(mut line: &str) -> Result<String, ChunkParseError> {
        // remove comments
        line = utils::remove_comments(line);

        let tokens = line.trim().split_whitespace().collect::<Vec<&str>>();
        match tokens.len().cmp(&2) {
            Ordering::Greater => {
                Err(ChunkParseError {
                    chunk: Some(line.to_string()),
                    message: Some(
                        "accounts can't have spaces in them; use underscores instead: _".to_string(),
                    ),
                })
            },
            Ordering::Less => {
                Err(ChunkParseError {
                    chunk: Some(line.to_string()),
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
        match self.envelopes.get_mut(envelope.get_name()) {
            Some(e) => {
                e.merge(&envelope);
            }
            None => {
                self.envelopes.insert(envelope.get_name().to_string(), envelope);
            }
        }
    }
}
