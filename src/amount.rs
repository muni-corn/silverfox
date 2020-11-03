use crate::errors::*;
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

#[derive(Clone, Debug)]
pub struct Amount {
    pub mag: f64,
    pub symbol: Option<String>,
}

impl Amount {
    pub fn parse(s: &str, decimal_symbol: char) -> Result<Self, ParseError> {
        // reassign and remove double negatives
        let s = s.replace("--", "");

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
        let mut raw_mag = clump
            .chars()
            .filter(|&c| is_mag_char(c, decimal_symbol))
            .collect::<String>();

        if decimal_symbol != '.' {
            raw_mag = raw_mag.replace(decimal_symbol, ".");
        }

        let mag = match raw_mag.parse::<f64>() {
            Ok(m) => m,
            Err(_) => {
                return Err(ParseError {
                    message: Some(String::from("couldn't parse magnitude of amount")),
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

    /// Returns true if the char not a magnitude character, dot, or comma.
    fn is_symbol_char(c: char, decimal_symbol: char) -> bool {
        !is_mag_char(c, decimal_symbol) && c != '.' && c != ','
    }

    fn rounded_mag(&self) -> f64 {
        (self.mag * 100_000_000.0).round() / 100_000_000.0
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mag_fmt = if f.sign_plus() {
            format!("{:+}", self.rounded_mag())
        } else if self.mag < 0.0 {
            format!("{}", self.rounded_mag())
        } else {
            format!(" {}", self.rounded_mag())
        };

        if let Some(sym) = &self.symbol {
            if sym.len() <= 2 {
                write!(f, "{}{}", sym, mag_fmt)
            } else {
                write!(f, "{} {}", mag_fmt, sym)
            }
        } else {
            write!(f, "{}", mag_fmt)
        }
    }
}

impl Ord for Amount {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.symbol, other.symbol, "tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them.", self, other);

        self.mag
            .partial_cmp(&other.mag)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        assert_eq!(self.symbol, other.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, other);

        Some(self.cmp(other))
    }
}

impl PartialEq for Amount {
    fn eq(&self, other: &Self) -> bool {
        self.mag == other.mag && self.symbol == other.symbol
    }
}

impl Eq for Amount {}

/// + operator
impl Add for Amount {
    type Output = Self;

    fn add(mut self, rhs: Amount) -> Self::Output {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

        self.mag += rhs.mag;

        self
    }
}

/// += operator
impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Amount) {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

        self.mag += rhs.mag;
    }
}

/// - operator
impl Sub for Amount {
    type Output = Self;

    fn sub(self, rhs: Amount) -> Self::Output {
        self.add(-rhs)
    }
}

/// -= operator
impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Amount) {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

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

/// + operator
impl Add<&Amount> for Amount {
    type Output = Self;

    fn add(mut self, rhs: &Amount) -> Self::Output {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

        self.mag += rhs.mag;

        self
    }
}

/// += operator
impl AddAssign<&Amount> for Amount {
    fn add_assign(&mut self, rhs: &Amount) {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

        self.mag += rhs.mag;
    }
}

/// - operator
impl Sub<&Amount> for Amount {
    type Output = Self;

    fn sub(mut self, rhs: &Amount) -> Self::Output {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

        self.mag -= rhs.mag;

        self
    }
}

/// -= operator
impl SubAssign<&Amount> for Amount {
    fn sub_assign(&mut self, rhs: &Amount) {
        assert_eq!(self.symbol, rhs.symbol,"tried to operate on two amounts with differing symbols: {} and {}. developers should check for non-matching Amount symbols before performing operations on them." , self, rhs);

        self.mag -= rhs.mag;
    }
}

/// AmountPool is a collection of amounts, possibly with different currencies. AmountPool is
/// designed to assist with handling these different amounts of different currencies
#[derive(Clone, Debug)]
pub struct AmountPool {
    pool: Vec<Amount>,
}

impl AddAssign<Amount> for AmountPool {
    fn add_assign(&mut self, amount: Amount) {
        *self += &amount
    }
}

impl Add<&Amount> for AmountPool {
    type Output = Self;

    fn add(mut self, amount: &Amount) -> Self::Output {
        let mut iter = self.pool.iter_mut();
        match iter.find(|a| a.symbol == amount.symbol) {
            Some(a) => {
                *a += amount;
            }
            None => {
                self.pool.push(amount.clone());
            }
        }

        self
    }
}

impl Sub<Amount> for AmountPool {
    type Output = Self;

    fn sub(self, amount: Amount) -> Self::Output {
        self - &amount
    }
}

impl AddAssign<&Amount> for AmountPool {
    fn add_assign(&mut self, amount: &Amount) {
        let mut iter = self.pool.iter_mut();
        match iter.find(|a| a.symbol == amount.symbol) {
            Some(a) => {
                *a += amount;
            }
            None => {
                self.pool.push(amount.clone());
            }
        }
    }
}

impl Sub<&Amount> for AmountPool {
    type Output = Self;

    fn sub(self, amount: &Amount) -> Self::Output {
        self + &(-amount.clone())
    }
}

impl SubAssign<&Amount> for AmountPool {
    fn sub_assign(&mut self, amount: &Amount) {
        let mut iter = self.pool.iter_mut();
        match iter.find(|a| a.symbol == amount.symbol) {
            Some(a) => {
                *a -= amount;
            }
            None => {
                self.pool.push(-amount.clone());
            }
        }
    }
}

impl AddAssign<&AmountPool> for AmountPool {
    fn add_assign(&mut self, other: &AmountPool) {
        for amount in other.iter() {
            *self += amount;
        }
    }
}

impl AddAssign<AmountPool> for AmountPool {
    fn add_assign(&mut self, other: AmountPool) {
        *self += &other
    }
}

impl IntoIterator for AmountPool {
    type Item = Amount;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pool.into_iter()
    }
}

impl AmountPool {
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Not to be confused with `is_zero`, `is_empty` returns true if and only if there are no
    /// amounts contained within this pool.
    ///
    /// If there is one or more amounts with zero magnitude, this function returns false because
    /// there are still amounts being tracked within this pool.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn new() -> Self {
        Default::default()
    }

    pub fn only(&self, symbol: &Option<String>) -> Amount {
        self.pool
            .iter()
            .find(|a| a.symbol == *symbol)
            .unwrap_or(&Amount::zero())
            .clone()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Amount> {
        self.pool.iter()
    }

    /// Returns true if either (a) the pool is empty, or (b) all amounts in the pool have zero
    /// magnitiude.
    pub fn is_zero(&self) -> bool {
        if self.is_empty() {
            return true
        }

        for amt in &self.pool {
            if amt.mag != 0.0 {
                return false
            }
        }

        true
    }
}

impl Default for AmountPool {
    fn default() -> Self {
        Self { pool: Vec::new() }
    }
}

impl From<Amount> for AmountPool {
    fn from(amount: Amount) -> Self {
        Self { pool: vec![amount] }
    }
}

impl fmt::Display for AmountPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.len() {
            0 => Ok(()),
            1 => write!(f, "{}", self.pool[0]),
            _ => {
                for a in self.pool.iter() {
                    write!(f, "\n\t{}", a)?;
                }

                Ok(())
            }
        }
    }
}

/// Returns true if the char is a digit, decimal symbol, or dash.
fn is_mag_char(c: char, decimal_symbol: char) -> bool {
    c.is_digit(10) || c == decimal_symbol || c == '-'
}
