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
use errors::{MvelopesError, BasicError};
use flags::{CommandFlags, Subcommand};
use std::convert::TryFrom;
use ledger::Ledger;

fn main() {
    match parse_flags() {
        Ok(f) => {
            if let Err(e) = execute_flags(f) {
                eprintln!("{}", e)
            }
        },
        Err(e) => {
            eprintln!("{}", e)
        }
    }

    // parse ledger
    // let ledger;
    // match ledger::Ledger::from_file(&ledger_path_opt.unwrap()) {
    //     Ok(l) => {
    //         ledger = l;
    //     },
    //     Err(e) => {
    //         eprintln!("{}", e);
    //         return
    //     }
    // };

    // `envelopes` command
    // ledger.display_envelopes();

    // `balance` command
    // if let Err(e) = ledger.display_flat_balance() {
    //     eprintln!("{}", e);
    // }
}

// fn get_mvelopes_file() -> Result<String, String> {
//     // attempt to get from env variable $MVELOPES_FILE first
//     match env::var("MVELOPES_FILE") {
//         Ok(f) => Ok(f),
//         Err(e) => {
//             println!("$MVELOPES_FILE variable probably doesn't exist ('{}')", e);

//             // or attempt to get sibling file to $LEDGER_FILE
//             match get_ledger_path() {
//                 Ok(p) => {
//                     let parent_dir_path = p.parent();
//                     let mvelopes_sibling_path = parent_dir_path.join("mvelopes.journal");

//                     match mvelopes_sibling_path.to_str() {
//                         Some(p) => Ok(String::from(p)),
//                         None => Err(String::from("Getting sibling path didn't work"))
//                     }
//                 },
//                 Err(e) => {
//                     Err(format!("mvelopes file must be provided manually, due to '{}'", e))
//                 }
//             }
//         }
//     }
// }

fn parse_flags() -> Result<CommandFlags, BasicError> {
    let mut args = env::args();

    let subcommand: Subcommand;
    let mut file_path: Option<PathBuf> = None;
    let mut no_move = false;
    let mut csv_file = None;
    let mut rules_file = None;

    // parse subcommand
    match args.nth(1) {
        Some(a) => {
            subcommand = Subcommand::try_from(a.as_str())?;
        },
        None => {
            display_help();
            std::process::exit(0);
        }
    }

    while let Some(arg) = args.next() {
        // match boolean flags first
        match arg.as_str() {
            "--no-move" | "-n" => {
                no_move = true;
            },
            _ => {
                // then flags that require arguments
                let arg_value = parse_argument_value(args.next(), &arg)?;
                match arg.as_str() {
                    "-f" | "--file" => {
                        file_path = Some(PathBuf::from(arg_value));
                    },
                    "--csv-file" | "--csv" => {
                        csv_file = Some(PathBuf::from(arg_value));
                    },
                    "--rules-file" | "--rules" => {
                        rules_file = Some(PathBuf::from(arg_value));
                    },
                    _ => {
                        return Err(BasicError {
                            message: format!("mvelopes doesn't recognize this flag: `{}`", arg)
                        })
                    }
                }
            }
        }
    }


    if let Some(path) = file_path {
        Ok(CommandFlags {
            file_path: path,
            subcommand,
            no_move,
            csv_file,
            rules_file,
        })
    } else {
        // if flags.file_path is still empty after parsing flags, try to get it from the environment
        // variable
        match get_ledger_path() {
            Some(path) => {
                Ok(CommandFlags {
                    file_path: path,
                    subcommand,
                    no_move,
                    csv_file,
                    rules_file,
                })
            }
            None => {
                Err(BasicError::new("mvelopes wasn't given a file path. you can specify one with the `-f` flag or by setting the $LEDGER_FILE environment variable"))
            }
        }
    }
}

fn parse_argument_value(arg: Option<String>, name: &str) -> Result<String, BasicError> {
    match arg {
        Some(a) => Ok(a),
        None => Err(BasicError {
            message: format!("no value was supplied for the argument `{}`", name)
        })
    }
}

fn display_help() {
    println!("hello! i'm mvelopes! i tend to refer to myself in third person.");
    println!("you can use one of the subcommands to get information about your journal:");
    println!("    (b)alance      display all accounts and their respective values");
    println!("    (e)nvelopes    view your envelopes and how much is saved up in each");
    println!("    (r)egister     list all transactions");
    println!("    (a)dd          add a new transaction to your journal");
    println!("    (i)mport       parse entries from a csv file and add them to your journal");
    // println!();
    // println!("you can get more information about each subcommand with the --help flag, like so:");
    // println!("    mvelopes b --help")
}

fn get_ledger_path() -> Option<PathBuf> {
    match env::var("LEDGER_FILE") {
        Ok(v) => Some(PathBuf::from(v)),
        Err(_) => None
    }
}

fn execute_flags(flags: CommandFlags) -> Result<(), MvelopesError> {
    let mut ledger = match Ledger::from_file(&flags.file_path) {
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
                    return Err(MvelopesError::from(BasicError {
                        message: String::from("if you're importing a csv file, you need to specify the csv file with the --csv flag")
                    }))
                },
            }
        }
        _ => return Err(MvelopesError::from(BasicError {
            message: format!("the `{}` subcommand is recognized by mvelopes, but not supported yet. sorry :(", flags.subcommand)
        })),
    }

    Ok(())
}
