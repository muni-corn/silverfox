use crate::ledger::errors::*;
use crate::ledger::utils;
use crate::ledger::Posting;
use crate::ledger::Amount;
use std::collections::HashSet;
use std::fmt;

pub enum EntryStatus {
    /// `?`
    Pending,
    /// `!`
    Cleared,
    /// `*`
    Reconciled,
}

impl fmt::Display for EntryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            EntryStatus::Reconciled => '*',
            EntryStatus::Cleared => '!',
            EntryStatus::Pending => '?'
        };

        write!(f, "{}", symbol)
    }
}

pub struct Entry {
    date: chrono::NaiveDate,
    status: EntryStatus,
    description: String,
    payee: Option<String>,
    pub postings: Vec<Posting>,

    /// True if the Entry has a posting with a blank amount. Must be kept private to prevent any
    /// mutations from outside the struct.
    has_blank_posting: bool,

    /// True if the Entry has more than one type of currency throughout. Private so that external
    /// code can't modify this boolean. It is set in the parsing process of an Entry.
    mixed_currencies: bool,

    /// True if the Entry has a posting for an envelope. Private so that external code can't modify
    /// this boolean. It is set in the parsing process of an Entry.
    has_envelope_posting: bool,
}

impl Entry {
    pub fn parse(
        chunk: &str,
        date_format: &str,
        decimal_symbol: char,
        accounts: &HashSet<&String>,
    ) -> Result<Self, MvelopesError> {
        let trimmed_chunk = chunk.trim();
        if trimmed_chunk.is_empty() {
            return Err(ParseError {
                context: None,
                message: Some("entry to parse is completely empty. this is an error with mvelopes's programming. please report it!".to_string()),
            }.into());
        }

        let mut lines = trimmed_chunk.lines();

        // parse the header. parse_header returns the entry to start with
        let mut entry = if let Some(l) = lines.next() {
            Self::parse_header(l, date_format)?
        } else {
            let err = ParseError::new().set_context(chunk).set_message("header couldn't be parsed because it doesn't exist. this is an error with mvelopes's programming. please report it!");
            return Err(MvelopesError::from(err));
        };

        let mut symbol_set: HashSet<String> = HashSet::new();

        // parse postings
        for raw_posting in lines {
            // if blank, skip
            if raw_posting.trim().is_empty() {
                continue;
            }

            match Posting::parse(raw_posting, decimal_symbol, &accounts) {
                Ok(p) => {
                    if p.get_amount().is_none() {
                        // this entry now has a blank posting. this must be set so that
                        // get_blank_amount knows whether to return None or Some
                        entry.has_blank_posting = true;
                    } else if let Some(s) = &p.get_amount().as_ref().unwrap().symbol { // unwrap is safe here since the last if statement failed
                        // this block shouldn't execute if the posting is blank, which is why this
                        // section is in an 'else' block
                        // if the symbol to the posting's amount exists, add it to the symbol set
                        symbol_set.insert(s.to_owned());
                    } else {
                        // if the symbol doesn't exist, add the blank symbol
                        symbol_set.insert("".to_string());
                    }

                    if p.get_envelope_name().is_some() {
                        // this entry now has a blank posting. this must be set so that
                        // get_blank_amount knows whether to return None or Some

                        entry.has_envelope_posting = true;
                    }

                    // push the posting
                    entry.postings.push(p);
                },
                Err(e) => return Err(MvelopesError::from(e)),
            }
        }

        // parsing all postings is done. if there are multiple currencies involved in this entry,
        // set the boolean as such
        
        if symbol_set.len() > 1 {
            entry.mixed_currencies = true;
        }

        // validate this entry
        if let Err(e) = entry.validate(chunk) {
            return Err(MvelopesError::from(e));
        }

        Ok(entry)
    }

    fn parse_header(header: &str, date_format: &str) -> Result<Self, ParseError> {
        let clean_header = utils::remove_comments(header);
        let header_tokens = clean_header.split_whitespace().collect::<Vec<&str>>();

        if header_tokens.is_empty() {
            return Err(ParseError::new().set_message("couldn't parse an entry header because it's blank. this is an error with mvelopes's programming; please report it!"));
        }

        // parse date
        let date = match chrono::NaiveDate::parse_from_str(header_tokens[0], date_format) {
            Ok(d) => d,
            _ => {
                let message = format!(
                    "couldn't parse date `{}` with format `{}`",
                    header_tokens[0], date_format
                );
                return Err(ParseError {
                    message: Some(message),
                    context: Some(clean_header.to_string()),
                });
            }
        };

        // parse status
        let status = match header_tokens[1] {
            "?" => EntryStatus::Pending,
            "!" => EntryStatus::Cleared,
            "*" => EntryStatus::Reconciled,
            _ => return Err(ParseError {
                message: Some(format!("mvelopes requires statuses on entries and `{}` is not a status that mvelopes understands", header_tokens[1])),
                context: Some(clean_header.to_string())
            })
        };

        // parse description_and_payee
        let description_and_payee: &str = &header_tokens[2..].join(" ");
        let (description, payee) = if let Some(i) = description_and_payee.find('[') {
            if let Some(j) = description_and_payee.rfind(']') {
                // both brackets exist, so take everything before the opening bracket as the
                // description and everything in the brackets as the payee
                // yeah, this means anything after the closing bracket won't be included :/
                let d = description_and_payee[..i].trim().to_string();
                let p = description_and_payee[i + 1..j].trim().to_string();

                (d, Some(p))
            } else {
                // only opening bracket exists, and that's kind of an issue
                return Err(ParseError::new().set_message("mvelopes wanted to parse a payee in this header, but couldn't because it wasn't given a closing square bracket: ]").set_context(header));
            }
        } else {
            (description_and_payee.to_string(), None)
        };

        Ok(Entry {
            payee,
            description,
            date,
            status,
            postings: Vec::new(),
            has_blank_posting: false,
            mixed_currencies: false,
            has_envelope_posting: false,
        })
    }

    pub fn get_blank_amount(&self) -> Result<Option<Amount>, ProcessingError> {
        if !self.has_blank_posting {
            // return None if the Entry has no blank amount
            Ok(None)
        } else {
            let mut blank_amount = Amount::zero();

            // calculation of the blank amount depends on whether or not multiple currencies exist
            if self.mixed_currencies {
                // if multiple currencies exist, attempt to return the sum of the native amounts.
                // if any of the native amounts are None, the calculation fails and this function
                // returns an error
                let mut native_blank_amount = 0.0;
                for posting in &self.postings {
                    match posting.get_native_value() {
                        Some(v) => native_blank_amount -= v,
                        None => {
                            // native_value will be None for the blank amount, so only throw an
                            // error if the posting's amount is Some
                            if posting.get_amount().is_some() {
                                let err = ProcessingError::new().set_message("mvelopes couldn't calculate a value for an entry's blank posting amount. there are multiple currencies in this entry, but one posting does not provide its currency's worth in your native currency.").set_context(&self.display());
                                return Err(ProcessingError::from(err))
                            }
                        }
                    }
                }

                Ok(Some(Amount {
                    mag: native_blank_amount,
                    symbol: None
                }))
            } else {
                // for each posting, subtract that posting's amount from the blank amount (as long as
                // `posting` doesn't have a blank amount)
                for posting in &self.postings {
                    if let Some(a) = posting.get_amount() {
                        blank_amount -= a.clone();
                    }
                }

                Ok(Some(blank_amount))
            }
        }
    }

    /// Checks that the Entry is valid. Returns a ValidationError if it is invalid. An Entry is
    /// valid when all of the following are true:
    ///
    /// - it contains no more than one blank posting amount
    /// - it's balanced (the sum of its postings equals zero)
    /// - it contains no more than one type of currency when a blank posting amount exists (later
    ///   to be supported)
    fn validate(&self, context: &str) -> Result<(), ValidationError> {
        let mut blank_amounts = 0;
        let mut symbol_set = HashSet::new();
        for posting in &self.postings {
            // does amount exist?
            if let Some(a) = posting.get_amount() {
                // if so, add its symbol to the set if it exists
                if let Some(s) = &a.symbol {
                    symbol_set.insert(s);
                }
            } else {
                blank_amounts += 1;

                // if more than one blank amount, quit here and throw an error
                if blank_amounts > 1 {
                    return Err(ValidationError::new()
                        .set_message("a single entry can't have more than one blank posting")
                        .set_context(context));
                }
            }
        }

        // if there's a blank amount but the currencies aren't consistent, we can't infer the
        // blank's amount; there's a way around this that will be worked out in the future, but for
        // now it will be unsupported: TODO
        if blank_amounts > 0 && symbol_set.len() > 1 {
            return Err(ValidationError::new().set_message("mvelopes can't infer the amount of a blank posting when other postings have mixed currencies").set_context(context));
        }

        Ok(())
    }

    pub fn display(&self) -> String {
        let payee = if let Some(p) = &self.payee {
            p
        } else {
            "No payee"
        };

        let mut s = format!("{} {} {} [{}]", self.date, self.status, self.description, payee);
        for posting in &self.postings {
            s.push_str(&format!("\n{}", posting));
        }

        s
    }

    pub fn has_envelope_postings(&self) -> bool {
        self.has_envelope_posting
    }

    pub fn get_envelope_postings(&self) -> Vec<Posting> {
        let mut clone = self.postings.clone();
        clone.retain(|p| p.get_envelope_name().is_some());
        clone
    }

    pub fn get_date(&self) -> &chrono::NaiveDate {
        &self.date
    }

    pub fn has_blank_posting(&self) -> bool {
        self.has_blank_posting
    }

    pub fn contains_account_posting(&self, account_name: &str) -> bool {
        self.postings.iter().find(|&p| p.get_account() == account_name).is_some()
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY_STR: &'static str = 
        "2019/08/02 * Groceries [Grocery store]
            accounts:assets:checking -50
            expenses:groceries        50";

    #[test]
    fn test_parse() {
        Entry::parse(ENTRY_STR, "%Y/%m/%d", '.', accounts);
    }
}
