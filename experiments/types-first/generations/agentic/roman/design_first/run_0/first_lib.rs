use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange(u32),
    Empty,
    InvalidChar(char),
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub fn to_roman(_n: u32) -> Result<String, RomanError> {
    todo!()
}

pub fn from_roman(_s: &str) -> Result<u32, RomanError> {
    todo!()
}
