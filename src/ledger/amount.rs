use std::ops::{Add, AddAssign, SubAssign, Neg};
use crate::ledger::errors::*;
use std::fmt;

#[derive(Clone)]
pub struct Amount {
    pub mag: f64,
    pub symbol: Option<String>,
}

impl Amount {
    pub fn parse(s: &str, decimal_symbol: char) -> Result<Self, ParseError> {
        let split = s.split_whitespace().collect::<Vec<&str>>();

        let clump = match split.len() {
            2 => split.join(" "),
            1 => split[0].to_string(),
            _ => {
                return Err(ParseError {
                    context: Some(s.to_string()),
                    message: Some("this amount isn't valid".to_string()),
                })
            }
        };

        // parse amount and currency in the same chunk
        // parse magnitude
        let raw_mag = clump
            .chars()
            .filter(|&c| Self::is_mag_char(c, decimal_symbol))
            .collect::<String>();
        let mag = match raw_mag.parse::<f64>() {
            Ok(m) => m,
            Err(_) => {
                return Err(ParseError {
                    message: Some(format!("couldn't parse magnitude of amount; {}", raw_mag)),
                    context: Some(s.to_string()),
                })
            }
        };

        // parse symbol
        let raw_sym = clump
            .chars()
            .filter(|&c| Self::is_symbol_char(c, decimal_symbol))
            .collect::<String>();
        let trimmed_raw_sym = raw_sym.trim();
        let symbol = match trimmed_raw_sym.len() {
            0 => None,
            _ => Some(trimmed_raw_sym.to_string()),
        };

        Ok(Self { mag, symbol })
    }

    /// Returns a blank amount without a symbol.
    pub fn zero() -> Self {
        Amount {
            mag: 0.0,
            symbol: None,
        }
    }

    /// Returns true if the char is a digit, decimal symbol, or dash.
    fn is_mag_char(c: char, decimal_symbol: char) -> bool {
        c.is_digit(10) || c == decimal_symbol || c == '-'
    }

    /// Returns true if the char not a magnitude character, dot, or comma.
    fn is_symbol_char(c: char, decimal_symbol: char) -> bool {
        !Self::is_mag_char(c, decimal_symbol) && c != '.' && c != ','
    }

    pub fn display(&self) -> String {
        let mag_fmt: String = if self.mag < 0.0 {
            format!("{}", (self.mag*100_000_000.0).round() / 100_000_000.0)
        } else {
            format!(" {}", (self.mag*100_000_000.0).round() / 100_000_000.0)
        };

        if let Some(s) = &self.symbol {
            if s.len() <= 2 {
                format!("{}{}", s, mag_fmt)
            } else {
                format!("{} {}", mag_fmt, s)
            }
        } else {
            format!("{}", mag_fmt)
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }

}

/// + operator
impl Add<&Amount> for Amount {
    type Output = Self;

    fn add(mut self, rhs: &Amount) -> Self::Output {
        if self.symbol != rhs.symbol {
            panic!("tried to add two amounts with differing symbols: {} and {}", self, rhs);
        }

        self.mag += rhs.mag;

        self
    }
}

/// += operator
impl AddAssign<&Amount> for Amount {
    fn add_assign(&mut self, rhs: &Amount) {
        if self.symbol != rhs.symbol {
            panic!("tried to add two amounts with differing symbols: {} and {}", self, rhs);
        }

        self.mag += rhs.mag;
    }
}

/// -= operator
impl SubAssign<&Amount> for Amount {
    fn sub_assign(&mut self, rhs: &Amount) {
        if self.symbol != rhs.symbol {
            panic!("tried to operate on two amounts with differing symbols: {} and {}", self, rhs);
        }

        self.mag -= rhs.mag;
    }
}

/// negation operator
impl Neg for Amount {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            mag: -self.mag,
            symbol: self.symbol,
        }
    }
}

/// AmountPool is a collection of amounts, possibly with different currencies. AmountPool is
/// designed to assist with handling these different amounts of different currencies
pub struct AmountPool {
    pool: Vec<Amount>,
}

impl AddAssign<&Amount> for AmountPool {
    fn add_assign(&mut self, amount: &Amount) {
        let mut iter = self.pool.iter_mut();
        match iter.find(|a| a.symbol == amount.symbol) {
            Some(a) => {
                *a += amount;
            },
            None => {
                self.pool.push(amount.clone());
            }
        }
    }
}

impl AmountPool {
    pub fn size(&self) -> usize {
        self.pool.len()
    }
}

impl From<Amount> for AmountPool {
    fn from(amount: Amount) -> Self {
        Self {
            pool: vec![amount]
        }
    }
}

impl fmt::Display for AmountPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.size() {
            0 => Ok(()),
            1 => write!(f, "{}", self.pool[0].display()),
            _ => {
                for a in self.pool.iter() {
                    write!(f, "\n\t{}", a.display())?;
                }

                Ok(())
            }
        }
    }
}
