use std::fmt;

#[derive(Debug)]
pub struct ChunkParseError {
    chunk: Option<String>,
    message: Option<String>,
}

impl fmt::Display for ChunkParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.message.is_some() && self.chunk.is_some() {
            write!(f, "couldn't parse the following chunk ({})\n\n{}", self.message.unwrap(), self.chunk.unwrap())
        } else if self.message.is_some() {
            write!(f, "couldn't parse a chunk: {}", self.message.unwrap())
        } else if self.chunk.is_some() {
            write!(f, "couldn't parse this chunk:\n{}", self.chunk.unwrap())
        } else {
            write!(f, "couldn't parse a chunk")
        }
    }
}
