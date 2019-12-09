use std::collections::HashSet;
use chrono;
use std::str::FromStr;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

pub enum CurrencyAlignment {
    Prefix,
    Postfix
}

pub enum PostingType {
    Real,
    BalancedVirtual,
    UnbalancedVirtual
}

pub struct Ledger {
    entries: Vec<Entry>,
    date_format: String, // default = "%Y/%m/%d"
    accounts: Option<HashSet<&'static str>>
}

pub struct Entry {
    date: chrono::Date<chrono::Local>,
    description: String,
    postings: Vec<Posting>
}

pub struct Posting {
    amount: Amount,
    account: String, native_price: Option<f64>,
    posting_type: PostingType,
    asserted_balance: Option<f64>
}

pub struct Amount {
    mag: f32,
    currency: String,
    currency_alignment: CurrencyAlignment
}

impl Ledger {
    pub fn from_file(file_path: String) {
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

        println!("{}", s);
    }
    // impl continued below with FromStr
}

// impl FromStr for Ledger {
    // fn from_str(s: &str) -> Result<Self, &'static str> {
    //     let lines = s.split("\n");
    //     for line in lines {
            
    //     }
    // }
// }

// impl FromStr for Entry {
    
// }

// impl FromStr for Posting {
    // fn from_str(s: &str) -> Result<Self, &'static str> {
        
    // }
// }
