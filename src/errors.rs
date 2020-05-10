use std::fmt;

// TODO auto-fixable errors

/// MvelopesError is an enum for all possible custom errors that mvelopes can throw. It is a
/// wrapper of sorts.
pub enum MvelopesError {
    Basic(BasicError),
    Parse(ParseError),
    Validation(ValidationError),
    Processing(ProcessingError),
    Io(std::io::Error),
    Csv(csv::Error),
}

impl From<BasicError> for MvelopesError {
    fn from(err: BasicError) -> Self {
        Self::Basic(err)
    }
}

impl From<ParseError> for MvelopesError {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<ValidationError> for MvelopesError {
    fn from(err: ValidationError) -> Self {
        Self::Validation(err)
    }
}

impl From<ProcessingError> for MvelopesError {
    fn from(err: ProcessingError) -> Self {
        Self::Processing(err)
    }
}

impl From<std::io::Error> for MvelopesError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<csv::Error> for MvelopesError {
    fn from(err: csv::Error) -> Self {
        Self::Csv(err)
    }
}

impl fmt::Display for MvelopesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MvelopesError::Basic(b) => b.fmt(f),
            MvelopesError::Validation(v) => v.fmt(f),
            MvelopesError::Parse(p) => p.fmt(f),
            MvelopesError::Processing(p) => p.fmt(f),
            MvelopesError::Io(o) => write!(f, "mvelopes encountered an i/o error: {}", o),
            MvelopesError::Csv(c) => c.fmt(f),
        }
    }
}

/// BasicError is a simple error with only a message that can be thrown at any time.
#[derive(Debug)]
pub struct BasicError {
    pub message: String
}

impl BasicError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string()
        }
    }
}

impl fmt::Display for BasicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// ParseError is thrown during the parsing phase of ledger construction. If mvelopes can't parse
/// something, this error type will be thrown.
#[derive(Debug)]
pub struct ParseError {
    pub context: Option<String>,
    pub message: Option<String>,
}

impl ParseError {
    /// Sets the message of the error, returning itself for the convenience of chaining.
    pub fn set_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());

        self
    }

    /// Sets the context (chunk) of the error, returning itself for the convenience of chaining.
    pub fn set_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());

        self
    }
}

impl Default for ParseError {
    /// Returns a fresh, blank ParseError.
    fn default() -> Self {
        ParseError { context: None, message: None }
    }

}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.message.is_some() && self.context.is_some() {
            write!(
                f,
                "mvelopes couldn't understand the following:\n\n{}\n\nmore information: {}",
                self.context.as_ref().unwrap(),
                self.message.as_ref().unwrap(),
            )
        } else if let Some(a) = &self.message {
            write!(f, "mvelopes ran across an issue in your journal: {}", a)
        } else if let Some(b) = &self.context {
            write!(f, "mvelopes couldn't understand this:\n\n{}\n\nbut no further information was provided", b)
        } else {
            write!(f, "something couldn't be parsed, but no information was provided")
        }
    }
}

/// ValidationError is thrown during the validation phase of ledger construction. If mvelopes finds
/// something that's invalid and can't continue with construction, this error type will be thrown.
#[derive(Debug)]
pub struct ValidationError {
    pub context: Option<String>,
    pub message: Option<String>,
}

impl ValidationError {
    /// Sets the message of the error, returning itself for the convenience of chaining.
    pub fn set_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());

        self
    }

    /// Sets the context (chunk) of the error, returning itself for the convenience of chaining.
    pub fn set_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());

        self
    }
}

impl Default for ValidationError {
    /// Returns a fresh, blank ValidationError.
    fn default() -> Self {
        ValidationError { context: None, message: None }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.message.is_some() && self.context.is_some() {
            write!(
                f,
                "the following is invalid to mvelopes:\n\n{}\n\nmore information: {}",
                self.context.as_ref().unwrap(),
                self.message.as_ref().unwrap(),
            )
        } else if let Some(a) = &self.message {
            write!(f, "mvelopes flagged your journal file as invalid: {}", a)
        } else if let Some(b) = &self.context {
            write!(f, "the following is invalid to mvelopes:\n\n{}\n\nbut no further information was provided", b)
        } else {
            write!(f, "mvelopes found something invalid, but no information was provided")
        }
    }
}

pub struct ProcessingError {
    pub context: Option<String>,
    pub message: Option<String>,
}

impl ProcessingError {
    /// Sets the message of the error, returning itself for the convenience of chaining.
    pub fn set_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());

        self
    }

    /// Sets the context (chunk) of the error, returning itself for the convenience of chaining.
    pub fn set_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());

        self
    }
}

impl Default for ProcessingError {
    /// Returns a fresh, blank ProcessingError.
    fn default() -> Self {
        ProcessingError { context: None, message: None }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.message.is_some() && self.context.is_some() {
            write!(
                f,
                "looks like your journal is all valid, but this couldn't be processed:\n\n{}\n\nmore information: {}",
                self.context.as_ref().unwrap(),
                self.message.as_ref().unwrap(),
            )
        } else if let Some(a) = &self.message {
            write!(f, "your journal is valid, but mvelopes couldn't process this: {}", a)
        } else if let Some(b) = &self.context {
            write!(f, "your journal is valid, but mvelopes couldn't process this:\n\n{}\n\nno further information was provided", b)
        } else {
            write!(f, "your journal is valid, but mvelopes couldn't process something (no information was provided)")
        }
    }
}
