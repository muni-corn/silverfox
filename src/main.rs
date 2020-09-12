pub mod account;
pub mod amount;
pub mod entry;
pub mod envelope;
pub mod errors;
pub mod flags;
pub mod importer;
pub mod ledger;
pub mod posting;
pub mod utils;

use std::env;
use std::path::PathBuf;
use errors::{SilverfoxError, BasicError};
use flags::{CommandFlags, Subcommand};
use ledger::Ledger;

fn main() {
    match flags::CommandFlags::parse_from_env() {
        Ok(f) => if let Err(e) = f.execute() {
            eprintln!("{}", e)
        },
        Err(e) => eprintln!("{}", e),
    }
}

fn get_file_from_env() -> Option<PathBuf> {
    if let Ok(v) = env::var("SILVERFOX_FILE") {
        Some(PathBuf::from(v))
    } else if let Ok(v) = env::var("LEDGER_FILE") {
        Some(PathBuf::from(v))
    } else {
        None
    }
}

fn execute_flags(flags: CommandFlags) -> Result<(), SilverfoxError> {
    let file_path = if let Some(f) = flags.file_path {
        f
    } else if let Some(e) = get_file_from_env() {
        e
    } else {
        return Err(SilverfoxError::Basic(BasicError::new("silverfox wasn't given a file to work with. there are a couple of ways you can do this:
    - use the `-f` flag from the command line (example: `silverfox -f ./path/to/file.sfox`)
    - set the environment variable $SILVERFOX_FILE or $LEDGER_FILE to a path to a file")))
    };

    let mut ledger = match Ledger::from_file(&file_path) {
        Ok(l) => l,
        Err(e) => return Err(e)
    };

    if !flags.no_move {
        if let Err(e) = ledger.fill_envelopes() {
            return Err(e)
        }
    }

    match flags.subcommand {
        Subcommand::Balance => ledger.display_flat_balance()?,
        Subcommand::Envelopes => ledger.display_envelopes(),
        Subcommand::Import => {
            match flags.csv_file {
                Some(c) => {
                    return ledger.import_csv(&c, flags.rules_file.as_ref())
                },
                None => {
                    return Err(SilverfoxError::from(BasicError {
                        message: String::from("if you're importing a csv file, you need to specify the csv file with the --csv flag")
                    }))
                },
            }
        }
        _ => return Err(SilverfoxError::from(BasicError {
            message: format!("the `{}` subcommand is recognized by silverfox, but not supported yet. sorry :(", flags.subcommand)
        })),
    }

    Ok(())
}
