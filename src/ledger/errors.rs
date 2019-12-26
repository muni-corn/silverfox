use std::fmt;

// TODO auto-fixable errors


/// MvelopesError is an enum for all possible custom errors that mvelopes can throw. It is a
/// wrapper of sorts.
pub enum MvelopesError {
    Parse(ParseError),
    Validation(ValidationError)
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

impl fmt::Display for MvelopesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MvelopesError::Validation(v) => v.fmt(f),
            MvelopesError::Parse(p) => p.fmt(f)
        }
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
    /// Returns a fresh, blank ParseError.
    pub fn new() -> Self {
        ParseError { context: None, message: None }
    }

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
    /// Returns a fresh, blank ValidationError.
    pub fn new() -> Self {
        Self { context: None, message: None }
    }

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
