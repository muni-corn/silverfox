use std::fmt;

// might use this later for auto-fixable errors
// #[derive(Debug)]
// pub trait AutoFixableError<T> {
//     type Result<T> = Result<T, std::error::Error>;
//
//     fn is_fixable() -> bool;
//     fn fix(T) -> Self::Result<T>;
//
//     fn prompt_user()
// }

#[derive(Debug)]
pub struct ChunkParseError {
    pub chunk: Option<String>,
    pub message: Option<String>,
}

impl ChunkParseError {
    pub fn new() -> Self {
        ChunkParseError { chunk: None, message: None }
    }

    pub fn set_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());

        self
    }

    pub fn set_chunk(mut self, chunk: &str) -> Self {
        self.chunk = Some(chunk.to_string());

        self
    }
}

impl fmt::Display for ChunkParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.message.is_some() && self.chunk.is_some() {
            write!(
                f,
                "mveloeps couldn't understand the following:\n\n\t{}\n\nmore information: {}",
                self.chunk.as_ref().unwrap(),
                self.message.as_ref().unwrap(),
            )
        } else if let Some(a) = &self.message {
            write!(f, "couldn't parse a chunk: {}", a)
        } else if let Some(b) = &self.chunk {
            write!(f, "couldn't parse this chunk:\n{}", b)
        } else {
            write!(f, "couldn't parse a chunk")
        }
    }
}
