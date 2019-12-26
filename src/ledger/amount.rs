pub struct Amount {
    mag: f64,
    symbol: Option<String>,
}

impl Amount {
    fn parse(s: &str, decimal_symbol: char) -> Result<Self, ParseError> {
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
        let symbol = match raw_sym.trim().len() {
            0 => None,
            _ => Some(raw_sym.trim().to_string()),
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
        let mag = if self.mag < 0.0 {
            format!("{}", self.mag)
        } else {
            format!(" {}", self.mag)
        };

        if let Some(s) = &self.symbol {
            if s.len() <= 2 {
                format!("{}{}", s, mag)
            } else {
                format!("{} {}", mag, s)
            }
        } else {
            format!("{}", mag)
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}
