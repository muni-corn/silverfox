use std::env;
use std::path::{PathBuf, MAIN_SEPARATOR};

mod ledger;

fn main() {
    // get the $LEDGER_FILE env var
    let ledger_file: String = get_ledger_file().unwrap();
    let mvelopes_file: String = get_mvelopes_file().unwrap();

    let ledger = ledger::Ledger::from_file(ledger_file);

    // parse ledger
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
                    let parent_dir_path = get_parent_dir_path(&f);
                    let mvelopes_sibling_path = parent_dir_path.join("mvelopes.journal");

                    match mvelopes_sibling_path.to_str() {
                        Some(p) => Ok(String::from(p)),
                        None => Err(String::from("Getting sibling path didn't work"))
                    }
                },
                Err(e) => {
                    Err(format!("mvelopes file must be provided manually, due to '{}'", e))
                }
            }
        }
    }
}

fn get_parent_dir_path(file_path: &String) -> PathBuf {
    let mut split: Vec<&str> = file_path.split(MAIN_SEPARATOR).collect();
    split.remove(split.len() - 1);

    let mut parent_path = PathBuf::from("/");
    for p in split {
        parent_path.push(p);
    }

    parent_path
}

fn get_ledger_file() -> Result<String, env::VarError> {
    env::var("LEDGER_FILE")
}
