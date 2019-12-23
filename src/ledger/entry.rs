use crate::ledger::errors::ChunkParseError;
use crate::ledger::Amount;
use crate::ledger::utils;

pub enum EntryStatus {
    Pending,    // ?
    Cleared,    // !
    Reconciled  // *
}

pub struct Entry {
    date: chrono::NaiveDate,
    status: EntryStatus,
    description: String,
    payee: Option<String>,
    postings: Vec<Posting>
}

pub struct Posting {
    amount: Option<Amount>,
    account: String,
    posting_type: PostingType,
    price_assertion: Option<Amount>,
    balance_assertion: Option<Amount>,
    total_balance_assertion: Option<Amount>,
    envelope_name: Option<String>       // if Some, it's an envelope posting
}

pub enum PostingType {
    Real,
    BalancedVirtual,    // unsupported
    UnbalancedVirtual,  // unsupported
}

impl Posting {
    fn new() -> Self {
        Self {
            amount: None,
            account: String::new(),
            posting_type: PostingType::Real,
            price_assertion: None,
            balance_assertion: None,
            total_balance_assertion: None,
            envelope_name: None
        }
    }

    pub fn parse(line: &str, decimal_symbol: char) -> Result<Self, ChunkParseError> {
        let mut posting = Self::new();

        // remove comments and other impurities
        let trimmed_line = utils::remove_comments(line).trim();
        let tokens = trimmed_line.split_whitespace().collect::<Vec<&str>>();

        let amount_tokens: Vec<&str>;
        match tokens[0] {
            "envelope" => {
                posting.account = tokens[1].to_string();
                posting.envelope_name = Some(tokens[2].to_string());
                amount_tokens = tokens[3..].to_vec();
            },
            _ => {
                posting.account = tokens[0].to_string();
                amount_tokens = tokens[1..].to_vec();
            }
        }

        if let Err(e) = posting.parse_amounts(&amount_tokens, decimal_symbol) {
            Err(e)
        } else {
            Ok(posting)
        }
    }

    fn parse_amounts(&mut self, amount_tokens: &[&str], decimal_symbol: char) -> Result<(), ChunkParseError> {
        self.balance_assertion = match Self::parse_balance_assertion_amount(amount_tokens, decimal_symbol) {
            Ok(a) => a,
            Err(e) => return Err(e)
        };

        self.total_balance_assertion = match Self::parse_total_balance_assertion_amount(amount_tokens, decimal_symbol) {
            Ok(a) => a,
            Err(e) => return Err(e)
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
                                if let Some(a) = &self.amount {
                                    // to determine price, we have to figure it out by dividing
                                    // the total cost by the original amount of this posting

                                    let calculated_price_amt = Amount {
                                        mag: total_cost.mag / a.mag,
                                        symbol: a.symbol.clone()
                                    };

                                    Some(calculated_price_amt)
                                } else { 
                                    return Err(ChunkParseError {
                                        message: Some("a total cost assertion can't be supplied if the posting has no amount".to_string()),
                                        chunk: None
                                    })
                                }
                            } else {
                                // nothing there? nothing will be used
                                None
                            }
                        },
                        Err(e) => return Err(e)
                    }
                }
            }
            Err(e) => return Err(e)
        };

        Ok(())
    }

    fn parse_balance_assertion_amount(amount_tokens: &[&str], decimal_symbol: char) -> Result<Option<Amount>, ChunkParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "!", |&s| s == "!!" || s == "@" || s == "=")
    }

    fn parse_total_balance_assertion_amount(amount_tokens: &[&str], decimal_symbol: char) -> Result<Option<Amount>, ChunkParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "!!", |&s| s == "!" || s == "@" || s == "=")
    }

    fn parse_price_amount(amount_tokens: &[&str], decimal_symbol: char) -> Result<Option<Amount>, ChunkParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "@", |&s| s == "!" || s == "!!" || s == "=")
    }

    fn parse_total_cost_amount(amount_tokens: &[&str], decimal_symbol: char) -> Result<Option<Amount>, ChunkParseError> {
        Self::extract_amount(amount_tokens, decimal_symbol, "=", |&s| s == "!" || s == "!!" || s == "@")
    }

    fn extract_amount<P>(amount_tokens: &[&str], decimal_symbol: char, wanted_operator: &str, unwanted_op_predicate: P) -> Result<Option<Amount>, ChunkParseError> where P: FnMut(&&str) -> bool {
        // find the balance_assertion token
        let mut iter = amount_tokens.iter();
        match iter.position(|&s| s == wanted_operator) {
            Some(i) => {
                // trim unwanted tokens
                let mut useful_tokens = &amount_tokens[i+1..];

                // find any other tokens that should be filtered out
                if let Some(i) = useful_tokens.iter().position(unwanted_op_predicate) {
                    // trim unwanted tokens
                    useful_tokens = &useful_tokens[..i];
                }

                // parse the amount
                match Amount::parse(useful_tokens.join(" ").as_str(), decimal_symbol) {
                    Ok(a) => Ok(Some(a)),
                    Err(e) => Err(e)
                }
            },
            None => {
                Ok(None)
            }
        }
    }
}

impl Entry {
    pub fn parse(chunk: &str, date_format: &str, decimal_symbol: char) -> Result<Self, ChunkParseError> {
        println!("parsing chunk: {}", chunk);

        let trimmed_chunk = chunk.trim();
        if trimmed_chunk.is_empty() {
            return Err(ChunkParseError {
                chunk: None,
                message: Some("entry to parse is completely empty. this is an error with mvelopes's programming. please report it!".to_string()),
            });
        }

        let mut lines = trimmed_chunk.lines();

        // parse the header. parse_header returns the entry to start with
        let mut entry = if let Some(l) = lines.nth(0) {
            Self::parse_header(l, date_format)?
        } else {
            let err = ChunkParseError::new().set_chunk(chunk).set_message("header couldn't be parsed because it doesn't exist. this is an error with mvelopes's programming. please report it!");
            return Err(err)
        };

        for raw_posting in lines.skip(1) {
            // if blank, skip
            if raw_posting.trim().is_empty() {
                continue;
            }

            match Posting::parse(raw_posting, decimal_symbol) {
                Ok(p) => entry.postings.push(p),
                Err(e) => return Err(e)
            }
        }

        Ok(entry)
    }

    fn parse_header(header: &str, date_format: &str) -> Result<Self, ChunkParseError> {
        let clean_header = utils::remove_comments(header);
        let header_tokens = clean_header.split_whitespace().collect::<Vec<&str>>();

        if header_tokens.is_empty() {
            return Err(ChunkParseError::new().set_message("couldn't parse an entry header because it's blank. this is an error with mvelopes's programming; please report it!"))
        }

        // parse date
        let date = match chrono::NaiveDate::parse_from_str(header_tokens[0], date_format) {
            Ok(d) => d,
            Err(_) => {
                let message = format!("couldn't parse date `{}` with format `{}`", header_tokens[0], date_format);
                return Err(ChunkParseError {
                    message: Some(message),
                    chunk: Some(clean_header.to_string())
                })
            }
        };

        // parse status
        let status = match header_tokens[1] {
            "?" => EntryStatus::Pending,
            "!" => EntryStatus::Cleared,
            "*" => EntryStatus::Reconciled,
            _ => return Err(ChunkParseError {
                message: Some(format!("mvelopes requires statuses on entries and `{}` is not a status that mvelopes understands", header_tokens[1])),
                chunk: Some(clean_header.to_string())
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
                let p = description_and_payee[i+1..j].trim().to_string();

                (d, Some(p))
            } else {
                // only opening bracket exists, and that's kind of an issue
                return Err(ChunkParseError {
                    message: Some("mvelopes wanted to parse a payee in this header, but couldn't because it wasn't given a closing square bracket (])".to_string()),
                    chunk: Some(header.to_string())
                })
            }
        } else {
            (description_and_payee.to_string(), None)
        };

        Ok(Entry {
            payee,
            description,
            date,
            status,
            postings: Vec::new()
        })
    }
}
