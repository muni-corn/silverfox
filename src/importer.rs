use crate::entry::{Entry, EntryStatus};
use crate::posting::Posting;
use crate::errors::*;
use std::collections::{HashMap, HashSet};
use std::collections::{LinkedList, VecDeque};
use std::path::{Path, PathBuf};
use std::fs;

pub struct CsvImporter {
    rules: Rules,
    records: VecDeque<csv::StringRecord>,
}

impl CsvImporter {
    pub fn from_file(csv_file: &Path) -> Result<Self, MvelopesError> {
        let rules_file = Self::get_sibling_rules_path(csv_file);

        Self::from_file_with_rules(csv_file, &rules_file)
    }

    fn get_sibling_rules_path(original: &Path) -> PathBuf {
        PathBuf::from(format!("{}.rules", original.display()))
    }

    pub fn from_file_with_rules(csv_file: &Path, rules_file: &Path) -> Result<Self, MvelopesError> {
        let mut reader = match csv::Reader::from_path(csv_file) {
            Ok(r) => r,
            Err(e) => return Err(MvelopesError::from(e)),
        };

        let rules = Rules::from_file(rules_file)?;
        let mut records: VecDeque<csv::StringRecord> = VecDeque::new();
        for result in reader.records().skip(rules.skip as usize) {
            match result {
                Ok(r) => records.push_back(r),
                Err(e) => return Err(MvelopesError::from(ParseError {
                    message: Some(format!("there was an error reading csv records: {}", e)),
                    context: None
                }))
            }
        }

        Ok(Self {
            rules,
            records,
        })
    }
}

impl Iterator for CsvImporter {
    type Item = Result<Entry, MvelopesError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.records.pop_front() {
            None => None,
            Some(r) => {
                Some(self.rules.get_entry_from_record(&r))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Rules {
    accounts: HashMap<String, String>,
    amount_strs: HashMap<String, String>,
    comment: String,
    description: String,
    date_format: String,
    date_str: String,
    decimal_symbol: char,
    fields: LinkedList<String>,
    payee: String,
    skip: i32,
    status: String,
    subrules: Vec<Subrules>,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            accounts: Default::default(),
            amount_strs: Default::default(),
            comment: Default::default(),
            date_format: String::from("%Y/%m/%d"),
            date_str: String::from("%date%"),
            decimal_symbol: '.',
            description: String::from("%description%"),
            fields: Default::default(),
            payee: String::new(),
            skip: 0,
            status: String::from("~"),
            subrules: Default::default(),
        }
    }
}

impl Rules {
    pub fn from_file(rules_file: &Path) -> Result<Self, MvelopesError> {
        let mut rules: Rules = Default::default();
        if let Err(e) = rules.add_from_file(rules_file) {
            Err(e)
        } else {
            Ok(rules)
        }
    }

    fn add_from_file(&mut self, rules_file: &Path) -> Result<(), MvelopesError> {
        let s = match fs::read_to_string(rules_file) {
            Ok(s) => s,
            Err(e) => return Err(MvelopesError::from(e)),
        };

        self.add_from_str(&s)
    }

    fn add_from_str(&mut self, s: &str) -> Result<(), MvelopesError> {
        // Some if parsing a Subrules. None if parsing other things.
        let mut parsing_subrules: Option<Subrules> = None;
        let default_rules: Rules = Default::default();

        for line in s.lines() {
            // if line contains nothing, continue
            if line.trim().is_empty() {
                continue
            }

            if let Some(s) = parsing_subrules.as_mut() {
                let mut chars = line.chars();
                if s.rules == default_rules {
                    // if parsing_subrules has no Rules (as in, it's equal to the Default), then...
                    if let Some(c) = chars.next() {
                        if c.is_whitespace() {
                            // a line starting with whitespace is a rule
                            (*s).rules.add_from_line(line)?;
                        } else {
                            // a line starting with a non-whitespace character is a pattern to the
                            // Subrules
                            (*s).patterns.push(String::from(line));
                        }
                    }

                    // don't parse any more (more parsing from here will cause unwanted changes to
                    // the root rules)
                    continue;
                } else {
                    // if parsing_subrules has Rules, then...
                    if let Some(c) = chars.next() {
                        if c.is_whitespace() {
                            // a line starting with whitespace is a rule
                            (*s).rules.add_from_line(line)?;

                            // don't parse any more (more parsing from here will cause unwanted changes to
                            // the root rules)
                            continue;
                        } else {
                            // a line starting with a non-whitespace character ends this parsing of
                            // these subrules
                            self.subrules.push(s.clone());
                            parsing_subrules = None;

                            // this is the only place we don't use the continue keyword. since this
                            // line triggers an ending to parsing Subrules, it could be something
                            // to be parsed in the root Rules
                        }
                    }
                }
            }

            if line.starts_with("if") {
                parsing_subrules = Some(Default::default());
                if let Some(i) = line.chars().position(|c| c.is_whitespace()) {
                    match parsing_subrules.as_mut() {
                        Some(s) => (*s).patterns.push(String::from(&line[i + 1..])),
                        None => unreachable!() // should be unreachable, as parsing_subrules was just initialized as Some
                    }
                }
            } else {
                self.add_from_line(line)?;
            }
        }

        // at the end, if parsing_subrules is Some, it needs to be added
        if let Some(r) = parsing_subrules {
            self.subrules.push(r);
        }

        Ok(())
    }

    fn add_from_line(&mut self, mut line: &str) -> Result<(), MvelopesError> {
        line = line.trim_start();

        let split_index = match line.chars().position(|c| c.is_whitespace()) {
            Some(i) => i,
            None => return Err(MvelopesError::from(ParseError::default().set_message("this rule has no value").set_context(line))),
        };

        // the first token is the rule name
        let rule_name = &line[..split_index];

        // the rest of the line is the value
        let rule_value = String::from(&line[split_index + 1..]);

        match rule_name {
            "fields" => {
                for field_name in rule_value.split(',') {
                    self.fields.push_back(String::from(field_name.trim()))
                }
            },
            "date_format" => self.date_format = rule_value,
            "date" => self.date_str = rule_value,
            "description" => self.description = rule_value,
            "payee" => self.description = rule_value,
            "comment" | "note" => self.comment = rule_value,
            "skip" => {
                self.skip = match rule_value.parse::<i32>() {
                    Ok(n) => n,
                    Err(e) => return Err(MvelopesError::from(ParseError {
                        message: Some(format!("the `skip` rule couldn't be parsed because of this error: {}", e)),
                        context: None,
                    })),
                }
            },
            "if" => {
                // because root-level if rules should be handled by add_from_str, this should not
                // be called unless rules are being added line by line, which is what happens when
                // parsing Subrules
                return Err(MvelopesError::from(ParseError::default().set_message("nested subrules aren't allowed")))
            },
            "include" | "use" => self.add_from_file(&PathBuf::from(rule_value))?,
            "decimal_symbol" | "decimal" => {
                if rule_value.len() > 1 {
                    return Err(MvelopesError::from(ParseError::default().set_message("decimal_symbol should be a single character").set_context(line)))
                } else {
                    self.decimal_symbol = rule_value.chars().next().unwrap();
                }
            },
            _ => {
                // attempt parsing an amount index or an account index
                if rule_name.starts_with("amount") {
                    let index_str = &rule_name["amount".len()..];
                    self.amount_strs.insert(String::from(index_str), rule_value);
                } else if rule_name.starts_with("account") {
                    let index_str = &rule_name["account".len()..];
                    self.accounts.insert(String::from(index_str), rule_value);
                } else {
                    return Err(MvelopesError::from(ParseError {
                        message: Some(format!("`{}` is not a rule that mvelopes understands", rule_name)),
                        context: Some(line.to_string()),
                    }))
                }
            }
        }

        Ok(())
    }

    pub fn get_entry_from_record(&self, record: &csv::StringRecord) -> Result<Entry, MvelopesError> {
        let mut variables: HashMap<String, String> = HashMap::new();

        for (field_name, field_value) in self.fields.iter().zip(record.iter()) {
            if variables.contains_key(field_name) {
                return Err(MvelopesError::from(ParseError {
                    message: Some(format!("there is a duplicate field definition in your rules file: `{}`", field_name)),
                    context: None,
                }))
            }

            variables.insert(String::from(field_name), String::from(field_value));
        }

        let raw_date = Self::inject_variables(&self.date_str, &variables);
        let date = match chrono::NaiveDate::parse_from_str(&raw_date, &self.date_format) {
            Ok(d) => d,
            Err(e) => return Err(MvelopesError::from(ParseError {
                message: Some(format!("there was an error parsing `{}` with the format `{}`: {}", raw_date, self.date_format, e)),
                context: None
            }))
        };
        let description = Self::inject_variables(&self.description, &variables);

        let status = Self::inject_variables(&self.status, &variables).parse::<EntryStatus>()?;

        let payee = if self.payee.trim().is_empty() {
            None
        } else {
            Some(Self::inject_variables(&self.payee, &variables))
        };

        let mut postings: Vec<Posting> = Vec::new();
        let mut account_set: HashSet<&String> = HashSet::new();
        for (index, account_name) in self.accounts.iter() {
            account_set.insert(&account_name);
            let to_parse = match self.amount_strs.get(index) {
                Some(amount_str) => {
                    format!("{} {}", account_name, amount_str)
                },
                None => {
                    account_name.clone()
                }
            };

            match Posting::parse(to_parse.as_str(), self.decimal_symbol, &account_set) {
                Ok(p) => postings.push(p),
                Err(e) => return Err(e),
            }
        }

        let comment = if self.comment.trim().is_empty() {
            None
        } else {
            Some(Self::inject_variables(&self.comment, &variables))
        };

        Ok(Entry::new(date, status, description, payee, postings, comment))
    }

    fn inject_variables(s: &str, variables: &HashMap<String, String>) -> String {
        let mut result = String::from(s);

        for (v_name, v_value) in variables.iter() {
            result = result.replace(format!("%{}%", v_name).as_str(), v_value);
        }

        result = result.replace("%%", "%"); // literal %

        result
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Subrules {
    patterns: Vec<String>,
    rules: Rules,
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES_STR: &str = "fields account, amount, currency, native_amount, native_usd

decimal_symbol .

date_format %Y.%m.%d

comment test comment

skip 1

if test0
    decimal_symbol ,
    amount 1,2

if test1
test2
test3
    amount 3.4
    comment new comment

if bad decimal
test4
test5
    amount 5.6
    decimal_symbol ,";

    #[test]
    fn sibling_rules_path_test() {
        let csv_path = PathBuf::from("csv_file.csv");
        let sibling = CsvImporter::get_sibling_rules_path(&csv_path);
        assert_eq!(sibling, PathBuf::from("csv_file.csv.rules"));
    }

    #[test]
    fn parse_rules_test() {
        let mut rules = Rules::default();
        let other = parse_rules_test_struct();
        if let Err(e) = rules.add_from_str(RULES_STR) {
            panic!("{}", e);
        } else {
            assert_eq!(rules, other);
        }
    }

    fn parse_rules_test_struct() -> Rules {
        let mut rules: Rules = Default::default();

        let mut subrules_0: Subrules = Default::default();
        subrules_0.rules.amount_strs.insert(String::new(), String::from("1,2"));
        subrules_0.rules.decimal_symbol = ',';
        subrules_0.patterns.push(String::from("test0"));

        let mut subrules_1: Subrules = Default::default();
        subrules_1.rules.amount_strs.insert(String::new(), String::from("3.4"));
        subrules_1.patterns.push(String::from("test1"));
        subrules_1.patterns.push(String::from("test2"));
        subrules_1.patterns.push(String::from("test3"));
        subrules_1.rules.comment = String::from("new comment");

        let mut bad_decimal_subrules: Subrules = Default::default();
        bad_decimal_subrules.rules.amount_strs.insert(String::new(), String::from("5.6"));
        bad_decimal_subrules.rules.decimal_symbol = ',';
        bad_decimal_subrules.patterns.push(String::from("bad decimal"));
        bad_decimal_subrules.patterns.push(String::from("test4"));
        bad_decimal_subrules.patterns.push(String::from("test5"));


        rules.fields.push_back(String::from("account"));
        rules.fields.push_back(String::from("amount"));
        rules.fields.push_back(String::from("currency"));
        rules.fields.push_back(String::from("native_amount"));
        rules.fields.push_back(String::from("native_usd"));
        rules.comment = String::from("test comment");
        rules.date_format = String::from("%Y.%m.%d");
        rules.decimal_symbol = '.';
        rules.skip = 1;
        rules.subrules.push(subrules_0);
        rules.subrules.push(subrules_1);
        rules.subrules.push(bad_decimal_subrules);

        rules
    }
}
