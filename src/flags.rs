use std::path::PathBuf;
use crate::errors;
use std::convert::TryFrom;

pub enum Subcommand {
    Balance,
    Envelopes,
    Register,
    Import,
    Add,
}

impl Subcommand {
    pub fn display(&self) -> String {
        String::from(match self {
            Self::Balance => "balance",
            Self::Envelopes => "envelopes",
            Self::Register => "register",
            Self::Import => "import",
            Self::Add => "add",
        })
    }
}

impl std::fmt::Display for Subcommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl TryFrom<&str> for Subcommand {
    type Error = errors::BasicError;

    fn try_from(s: &str) -> Result<Self, errors::BasicError> {
        if s.starts_with('b') {
            Ok(Self::Balance)
        } else if s.starts_with('e') {
            Ok(Self::Envelopes)
        } else if s.starts_with('r') {
            Ok(Self::Register)
        } else if s.starts_with('i') {
            Ok(Self::Import)
        } else if s.starts_with('a') {
            Ok(Self::Add)
        } else {
            Err(errors::BasicError {
                message: format!("`{}` is not a recognized subcommand. subcommands need to be the first argument made to silverfox. did you misplace your subcommand?", s)
            })
        }
    }
}

pub struct CommandFlags {
    pub file_path: PathBuf,
    pub subcommand: Subcommand,
    pub no_move: bool,
    pub csv_file: Option<PathBuf>,
    pub rules_file: Option<PathBuf>,
}
