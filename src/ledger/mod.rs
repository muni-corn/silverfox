use std::fmt;
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use entry::Entry;
use crate::ledger::errors::*;

pub mod entry;
pub mod envelope;
pub mod account;
pub mod errors;
pub mod utils;

pub struct Ledger {
    entries: Vec<Entry>,
    date_format: String, // default = "%Y/%m/%d"
    accounts: HashMap<String, account::Account>,
    default_currency: String,
    decimal_symbol: char
}

impl Ledger {
    fn blank() -> Self {
        Ledger {
            date_format: String::from("%Y/%m/%d"),
            entries: Vec::<Entry>::new(),
            accounts: HashMap::new(),
            default_currency: String::new(),
            decimal_symbol: '.'
        }
    }

    fn get_string_from_file(file_path: &Path) -> String {
        let path_display = file_path.display();
        let mut file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("couldn't open {}: {}", path_display, e)
        };

        let mut s = String::new();
        if let Err(e) = file.read_to_string(&mut s) {
            panic!("couldn't read {}: {}", path_display, e);
        } else {
            s
        }
    }

    pub fn from_file(file_path: &Path) -> Result<Self, MvelopesError> {
        let mut ledger = Self::blank();

        if let Err(e) = ledger.add_from_file(file_path) {
            Err(e)
        } else {
            Ok(ledger)
        }
    }

    fn add_from_file(&mut self, file_path: &Path) -> Result<(), MvelopesError> {
        let s = Self::get_string_from_file(file_path);

        if let Some(parent) = file_path.parent() {
            self.add_from_str(&s, parent)
        } else {
            panic!("a file without a valid parent can't be used")
        }
    }

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
                        return Err(e)
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

    fn parse_chunk(&mut self, chunk: &str, parent_path: &Path) -> Result<(), MvelopesError> {
        // a "chunk" starts at a line that starts with a non-whitespace
        // character and ends before the next line that starts with a
        // non-whitespace character
        
        if chunk.is_empty() {
            return Ok(()) // blank chunks are fine; they don't modify anything, so no error needed
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
            _ => self.parse_entry(chunk)
        }
    }

    fn set_currency(&mut self, cur: Option<&str>) -> Result<(), MvelopesError> {
        match cur {
            None => Err(MvelopesError::from(ParseError { 
                message: Some("no currency provided, but currency keyword was found".to_string()),
                context: None
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
                message: Some("no date format provided, but date_format keyword was found".to_string())
            })),
            Some(d) => {
                self.date_format = d.into();
                Ok(())
            }
        }
    }

    fn include(&mut self, file: Option<&str>, parent_path: &Path) -> Result<(), MvelopesError> {
        match file {
            None => Err(MvelopesError::from(ParseError::new().set_message("no file provided to an `include` clause"))),
            Some(f) => {
                self.add_from_file(&parent_path.join(f))
            }
        }
    }

    fn parse_entry(&mut self, chunk: &str) -> Result<(), MvelopesError> {
        match Entry::parse(chunk, &self.date_format, self.decimal_symbol) {
            Ok(entry) => {
                self.entries.push(entry);
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    fn parse_account(&mut self, chunk: &str) -> Result<(), MvelopesError> {
        match account::Account::parse(chunk, self.decimal_symbol, &self.date_format) {
            Ok(a) => {
                self.accounts.insert(a.get_name().to_string(), a);
                Ok(())
            },
            Err(e) => Err(MvelopesError::from(e))
        }
    }
}

impl fmt::Display for Ledger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ent in &self.entries {
            ent.fmt(f)?;
            write!(f, "\n")?;
        }

        Ok(())
    }
}

pub struct Amount {
    mag: f64,
    symbol: Option<String>
}

impl Amount {
    fn parse(s: &str, decimal_symbol: char) -> Result<Self, ParseError> {
        let split = s.split_whitespace().collect::<Vec<&str>>();

        let clump = match split.len() {
            2 => split.join(" "),
            1 => split[0].to_string(),
            _ => return Err(ParseError {
                context: Some(s.to_string()),
                message: Some("this amount isn't valid".to_string())
            })
        };

        // parse amount and currency in the same chunk
        // parse magnitude
        let raw_mag = clump.chars().filter(|&c| Self::is_mag_char(c, decimal_symbol)).collect::<String>();
        let mag = match raw_mag.parse::<f64>() {
            Ok(m) => m,
            Err(_) => return Err(ParseError {
                message: Some(format!("couldn't parse magnitude of amount; {}", raw_mag)),
                context: Some(s.to_string())
            })
        };

        // parse symbol
        let raw_sym = clump.chars().filter(|&c| Self::is_symbol_char(c, decimal_symbol)).collect::<String>();
        let symbol = match raw_sym.trim().len() {
            0 => None,
            _ => Some(raw_sym.trim().to_string())
        };

        Ok(Self {
            mag,
            symbol
        })
    }

    pub fn zero() -> Self {
        Amount {
            mag: 0.0,
            symbol: None
        }
    }

    // returns true if the char is a digit or decimal symbol
    fn is_mag_char(c: char, decimal_symbol: char) -> bool {
        c.is_digit(10) || c == decimal_symbol || c == '-'
    }

    fn is_symbol_char(c: char, decimal_symbol: char) -> bool {
        !Self::is_mag_char(c, decimal_symbol) && c != '.' && c != ','
    }

    pub fn display(&self) -> String {
        let mag = if self.mag < 0.0 {
            format!("{}", self.mag)
        } else {
            format!(" {}", self.mag)
        };

        if let Some(s) = &self.symbol {
            if s.len() <= 2 {
                format!("{}{}", s, mag)
            } else {
                format!("{} {}", mag, s)
            }
        } else {
            format!("{}", mag)
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}
