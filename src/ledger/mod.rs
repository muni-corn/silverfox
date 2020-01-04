use crate::ledger::amount::AmountPool;
use errors::*;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub mod account;
pub mod amount;
pub mod entry;
pub mod envelope;
pub mod errors;
pub mod posting;
pub mod utils;

pub type Account = account::Account;
pub type Amount = amount::Amount;
pub type Entry = entry::Entry;
pub type Envelope = envelope::Envelope;
pub type Posting = posting::Posting;

pub struct Ledger {
    entries: Vec<Entry>,
    date_format: String, // default = "%Y/%m/%d"
    accounts: HashMap<String, Account>,
    default_currency: String,
    decimal_symbol: char,
}

impl Ledger {
    /// Returns a blank ledger, with default values for `date_format` and `decimal_symbol`
    fn blank() -> Self {
        Ledger {
            date_format: String::from("%Y/%m/%d"),
            entries: Vec::<Entry>::new(),
            accounts: HashMap::new(),
            default_currency: String::new(),
            decimal_symbol: '.',
        }
    }

    /// Given a `file_path`, returns an entire file's contents as a String
    fn get_string_from_file(file_path: &Path) -> String {
        let path_display = file_path.display();
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("couldn't open {}: {}", path_display, e),
        };

        let mut s = String::new();
        if let Err(e) = file.read_to_string(&mut s) {
            panic!("couldn't read {}: {}", path_display, e);
        } else {
            s
        }
    }

    /// Returns a ledger parsed from a file at the `file_path`
    pub fn from_file(file_path: &Path) -> Result<Self, MvelopesError> {
        let mut ledger = Self::blank();

        if let Err(e) = ledger.add_from_file(file_path) {
            Err(e)
        } else {
            Ok(ledger)
        }
    }

    /// Adds to the ledger from the contents parsed from the file at the `file_path`
    fn add_from_file(&mut self, file_path: &Path) -> Result<(), MvelopesError> {
        let s = Self::get_string_from_file(file_path);

        if let Some(parent) = file_path.parent() {
            self.add_from_str(&s, parent)
        } else {
            panic!("a file without a valid parent can't be used")
        }
    }

    /// Adds to the ledger from the contents parsed from the string
    fn add_from_str(&mut self, s: &str, parent_path: &Path) -> Result<(), MvelopesError> {
        // init a chunk
        let mut chunk = String::new();

        // split lines
        let lines = s.lines();
        for mut line in lines {
            line = utils::remove_comments(line.trim_end());

            // if the first character of this line is whitespace, it is part of the current chunk.
            // if there is no first character, nothing happens
            if let Some(c) = line.chars().next() {
                if c.is_whitespace() {
                    chunk.push('\n');
                    chunk.push_str(line);
                } else {
                    if let Err(e) = self.parse_chunk(&chunk, parent_path) {
                        return Err(e);
                    }
                    chunk = String::from(line);
                }
            }
        }

        // parse the last chunk
        if let Err(e) = self.parse_chunk(&chunk, parent_path) {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Parses a single chunk and adds its contents to the ledger. Returns an MvelopesError is
    /// there was an issue in validation or in parsing.
    ///
    /// What is a "chunk"? A "chunk" starts at a line that starts with a non-whitespace character
    /// and ends before the next line that starts with a non-whitespace character.
    fn parse_chunk(&mut self, chunk: &str, parent_path: &Path) -> Result<(), MvelopesError> {
        if chunk.is_empty() {
            return Ok(()); // blank chunks are fine; they don't modify anything, so no error needed
        }

        let mut tokens = chunk.split_whitespace();
        let keyword = tokens.next();
        let value = tokens.next();
        match keyword {
            None => Ok(()),
            Some("account") => self.parse_account(chunk),
            Some("currency") => self.set_currency(value),
            Some("date_format") => self.set_date_format(value),
            Some("include") => self.include(value, parent_path),
            _ => self.parse_entry(chunk),
        }
    }

    /// Parses a currency symbol
    fn set_currency(&mut self, cur: Option<&str>) -> Result<(), MvelopesError> {
        match cur {
            None => Err(MvelopesError::from(ParseError {
                message: Some("no currency provided, but currency keyword was found".to_string()),
                context: None,
            })),
            Some(c) => {
                self.default_currency = c.into();
                Ok(())
            }
        }
    }

    fn set_date_format(&mut self, date_format: Option<&str>) -> Result<(), MvelopesError> {
        match date_format {
            None => Err(MvelopesError::from(ParseError {
                context: None,
                message: Some(
                    "no date format provided, but date_format keyword was found".to_string(),
                ),
            })),
            Some(d) => {
                self.date_format = d.into();
                Ok(())
            }
        }
    }

    fn include(&mut self, file: Option<&str>, parent_path: &Path) -> Result<(), MvelopesError> {
        match file {
            None => Err(MvelopesError::from(
                ParseError::new().set_message("no file provided to an `include` clause"),
            )),
            Some(f) => self.add_from_file(&parent_path.join(f)),
        }
    }

    fn parse_entry(&mut self, chunk: &str) -> Result<(), MvelopesError> {
        match Entry::parse(chunk, &self.date_format, self.decimal_symbol, &self.accounts.keys().collect()) {
            Ok(entry) => {
                for (_, account) in self.accounts.iter_mut() {
                    if let Err(e) = account.process_entry(&entry) {
                        return Err(MvelopesError::from(e))
                    }
                }
                self.entries.push(entry);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn parse_account(&mut self, chunk: &str) -> Result<(), MvelopesError> {
        match Account::parse(chunk, self.decimal_symbol, &self.date_format) {
            Ok(a) => {
                self.accounts.insert(a.get_name().to_string(), a);
                Ok(())
            }
            Err(e) => Err(MvelopesError::from(e)),
        }
    }

    pub fn display_flat_balance(&self) -> Result<(), MvelopesError> {
        let totals_map = match self.get_totals() {
            Ok(m) => m,
            Err(e) => return Err(e)
        };

        let mut totals_vec = totals_map.iter().collect::<Vec<(&String, &AmountPool)>>();
        totals_vec.sort_by(|a, b| a.0.cmp(b.0));

        for pair in totals_vec.iter() {
            println!("{:35}    {}", pair.0, pair.1);
        }

        Ok(())
    }

    fn get_totals(&self) -> Result<HashMap<String, AmountPool>, MvelopesError> {
        // map for account names to amount pools
        let mut totals_map: HashMap<String, AmountPool> = HashMap::new();

        // read: for each posting in the ledger, add its amount to its account in totals_map
        for entry in &self.entries {
            for posting in &entry.postings {
                let posting_amount = posting.get_amount();
                let posting_account = posting.get_account();
                // if the account key exists, just add to it. if it doesn't exist, insert a new key
                // with the amount
                match totals_map.get_mut(posting_account) {
                    Some(pool) => {
                        if let Some(a) = posting_amount {
                            *pool += a.clone();
                        } else { 
                            match entry.get_blank_amount() {
                                Ok(o) => {
                                    if let Some(b) = o {
                                        *pool += b;
                                    }
                                },
                                Err(e) => return Err(MvelopesError::from(e))
                            }
                        }
                    },
                    None => {
                        // if the posting amount exists, set an AmountPool from the amount as the
                        // key's value. otherwise, use an AmountPool from a zero Amount.
                        if let Some(a) = posting_amount {
                            totals_map.insert(posting_account.to_owned(), AmountPool::from(a.clone()));
                        } else {
                            totals_map.insert(posting_account.to_owned(), AmountPool::from(Amount::zero()));
                        }
                    }
                }
            }
        }

        Ok(totals_map)
    }

    pub fn display_envelopes(&self) {
        for (_, account) in self.accounts.iter() {
            account.display_envelopes();
        }
    }

    fn fill_envelopes(&self) {

    }
}

impl fmt::Debug for Ledger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ent in &self.entries {
            ent.fmt(f)?;
            write!(f, "\n")?;
        }

        Ok(())
    }
}
