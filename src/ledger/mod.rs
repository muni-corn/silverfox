use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use entry::Entry;
use std::ops::Add;

pub mod entry;
pub mod account;

pub struct Ledger {
    entries: Vec<Entry>,
    date_format: &'static str, // default = "%Y/%m/%d"
    accounts: HashMap<&'static str, account::Account>,
    default_currency: &'static str
}

pub struct Amount {
    mag: f32,
    currency: String
}

impl Ledger {
    pub fn from_file(file_path: String) -> Result<Self, &'static str> {
        let path = Path::new(&file_path);
        let path_display = path.display();
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => panic!("Couldn't open {}: {}", path_display, e)
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(e) => panic!("Couldn't read {}: {}", path_display, e),
            Ok(_) => ()
        }

        Ledger::from_str(&s)
    }

    fn from_str(s: &str) -> Result<Self, &'static str> {
        let mut ledger = Ledger {
            date_format: "%Y/%m/%d",
            entries: Vec::<Entry>::new(),
            accounts: HashMap::new(),
            default_currency: ""
        };

        let mut chunk = String::new();

        let lines = s.lines();
        for mut line in lines {
            line = line.trim_end();

            let first_char = line.chars().nth(0);
            if first_char.is_some() {
                if !first_char.unwrap().is_whitespace() {
                    ledger.parse_chunk(&chunk);
                    chunk = String::from(line);
                } else {
                    chunk.push('\n');
                    chunk.push_str(line);
                }
            }
        }

        // parse the last chunk
        ledger.parse_chunk(&chunk);

        Ok(ledger)
    }

    fn parse_chunk(&mut self, chunk: &String) {
        // a "chunk" starts at a line that starts with a non-whitespace
        // character and ends before the next line that starts with a
        // non-whitespace character

        println!("chunk: {}\n", chunk);
    }
}


