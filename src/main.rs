use std::env;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

mod ledger;

fn main() {
    // get the $LEDGER_FILE env var
    let ledger_file: String = get_ledger_file().unwrap();
    let mvelopes_file: String = get_mvelopes_file().unwrap();

    ledger::Ledger::from_file(ledger_file);

    // parse ledger
    // parse hledger output with assets and mvelopes
}

fn get_mvelopes_file() -> Result<String, String> {
    // attempt to get from env variable $MVELOPES_FILE first
    match env::var("MVELOPES_FILE") {
        Ok(f) => Ok(f),
        Err(e) => {
            println!("$MVELOPES_FILE probably doesn't exist ('{}')", e);

            // or attempt to get sibling file to $LEDGER_FILE
            match get_ledger_file() {
                Ok(f) => {
                    let parent_dir = get_parent_dir(f);
                    let mvelopes_sibling_path = Path::new(&parent_dir).join("mvelopes.journal");

                    match mvelopes_sibling_path.to_str() {
                        Some(p) => Ok(String::from(p)),
                        None => Err(String::from("Getting sibling path didn't work"))
                    }
                },
                Err(e) => {
                    Err(format!("Mvelopes file must be provided manually, due to '{}'", e))
                }
            }
        }
    }
}

fn get_parent_dir(file_path: String) -> String {
    let mut split: Vec<&str> = file_path.split(MAIN_SEPARATOR).collect();
    split.remove(split.len() - 1);

    let mut parent_path = PathBuf::from("/");
    for p in split {
        parent_path.push(p);
    }

    let path = parent_path.to_str().unwrap();

    String::from(path)
}

fn get_ledger_file() -> Result<String, env::VarError> {
    env::var("LEDGER_FILE")
}
