use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum JsonError {
    UnexpectedToken(String),
    UnexpectedEof,
    InvalidNumber(String),
    InvalidEscapeSequence(String),
    InvalidUnicodeSequence(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken(token) => write!(f, "Unexpected token: {}", token),
            JsonError::UnexpectedEof => write!(f, "Unexpected end of file"),
            JsonError::InvalidNumber(msg) => write!(f, "Invalid number: {}", msg),
            JsonError::InvalidEscapeSequence(seq) => write!(f, "Invalid escape sequence: {}", seq),
            JsonError::InvalidUnicodeSequence(seq) => {
                write!(f, "Invalid unicode sequence: {}", seq)
            }
        }
    }
}

impl Error for JsonError {}

pub type Result<T> = std::result::Result<T, JsonError>;
