use crate::errors::SilverfoxError;
use crate::ledger::Ledger;
use std::convert::TryFrom;
use std::env;
use std::path::PathBuf;

pub struct CommandFlags {
    pub file_path: Option<PathBuf>,
    pub subcommand: Subcommand,
    pub no_move: bool,

    pub csv_file: Option<PathBuf>,
    pub rules_file: Option<PathBuf>,

    pub other_accounts: bool,
    pub begin_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

impl CommandFlags {
    pub fn parse_from_env() -> Result<Self, SilverfoxError> {
        let mut args = env::args();

        // parse subcommand
        let subcommand = match args.nth(1) {
            Some(a) => Subcommand::try_from(a.as_str())?,
            None => {
                display_help();
                std::process::exit(0);
            }
        };

        let mut flags = CommandFlags {
            file_path: None,
            subcommand,
            no_move: false,
            csv_file: None,
            rules_file: None,
            other_accounts: false,
            begin_date: None,
            end_date: None,
        };

        while let Some(arg) = args.next() {
            // match boolean flags first
            match arg.as_str() {
                "--no-move" | "-n" => {
                    flags.no_move = true;
                }
                _ => {
                    // then flags that require arguments
                    let arg_value = parse_argument_value(args.next(), &arg)?;
                    match arg.as_str() {
                        "-f" | "--file" => {
                            flags.file_path = Some(PathBuf::from(arg_value));
                        }
                        "--csv-file" | "--csv" => {
                            flags.csv_file = Some(PathBuf::from(arg_value));
                        }
                        "--rules-file" | "--rules" => {
                            flags.rules_file = Some(PathBuf::from(arg_value));
                        }
                        _ => {
                            return Err(SilverfoxError::Basic(
                                format!(
                                    "silverfox doesn't recognize this flag: `{}`",
                                    arg
                                ),
                            ))
                        }
                    }
                }
            }
        }

        Ok(flags)
    }

    pub fn execute(&self) -> Result<(), SilverfoxError> {
        let file_path = if let Some(f) = &self.file_path {
            f.to_owned()
        } else if let Some(e) = get_file_from_env() {
            e
        } else {
            return Err(SilverfoxError::Basic(String::from("silverfox wasn't given a file to work with. there are a couple of ways you can do this:
    - use the `-f` flag from the command line (example: `silverfox -f ./path/to/file.sfox`)
    - set the environment variable $SILVERFOX_FILE or $LEDGER_FILE to a path to a file")));
        };

        let mut ledger = Ledger::from_file(&file_path)?;

        if !self.no_move {
            if let Err(e) = ledger.fill_envelopes() {
                return Err(e);
            }
        }

        match self.subcommand {
            Subcommand::Balance => ledger.display_flat_balance()?,
            Subcommand::Envelopes => ledger.display_envelopes(),
            Subcommand::Register => ledger.display_register(self.begin_date, self.end_date, None),
            Subcommand::Import => {
                match &self.csv_file {
                    Some(c) => {
                        return ledger.import_csv(&c, self.rules_file.as_ref())
                    },
                    None => {
                        return Err(SilverfoxError::Basic(String::from("if you're importing a csv file, you need to specify the csv file with the --csv flag")))
                    },
                }
            }
            // Subcommand::Register => ledger.display_register(self.period, self.begin_date, self.end_date),
            _ => return Err(SilverfoxError::Basic(format!("the `{}` subcommand is recognized by silverfox, but not supported yet. sorry :(", self.subcommand))),
        }

        Ok(())
    }
}

pub enum Subcommand {
    Summary,
    Balance,
    Envelopes,
    Register,
    Import,
    New,
}

impl Subcommand {
    pub fn display(&self) -> String {
        String::from(match self {
            Self::Summary => "summary",
            Self::Balance => "balance",
            Self::Envelopes => "envelopes",
            Self::Register => "register",
            Self::Import => "import",
            Self::New => "new",
        })
    }
}

impl std::fmt::Display for Subcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl TryFrom<&str> for Subcommand {
    type Error = SilverfoxError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Some(c) = s.chars().next() {
            match c {
                's' => Ok(Self::Summary),
                'b' => Ok(Self::Balance),
                'e' => Ok(Self::Envelopes),
                'r' => Ok(Self::Register),
                'i' => Ok(Self::Import),
                'n' => Ok(Self::New),
                _ =>
                    Err(SilverfoxError::Basic(format!("`{}` is not a recognized subcommand. subcommands need to be the first argument made to silverfox. did you misplace your subcommand?", s)))
            }
        } else {
            Err(SilverfoxError::Basic(format!("`{}` is not a recognized subcommand. subcommands need to be the first argument made to silverfox. did you misplace your subcommand?", s)))
        }
    }
}

fn display_help() {
    println!("hello! i'm silverfox!");
    println!("you can use one of the subcommands to get information about your journal:");
    println!("    (b)alance      display all accounts and their respective values");
    println!("    (e)nvelopes    view your envelopes and how much is saved up in each");
    println!("    (r)egister     list all transactions");
    println!("    (n)ew          add a new transaction to your journal");
    println!("    (i)mport       parse entries from a csv file and add them to your journal");
    // println!();
    // println!("you can get more information about each subcommand with the --help flag, like so:");
    // println!("    silverfox b --help")
}

fn parse_argument_value(arg: Option<String>, name: &str) -> Result<String, SilverfoxError> {
    match arg {
        Some(a) => Ok(a),
        None => Err(SilverfoxError::Basic(format!("no value was supplied for the argument `{}`", name))),
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
