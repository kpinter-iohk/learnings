use std::fmt;

/// Error returned by [`to_roman`] and [`from_roman`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    /// Input number was outside the supported range `1..=3999`.
    OutOfRange(u32),
    /// Input string was empty.
    Empty,
    /// Input string contained a character that is not one of `I V X L C D M`.
    InvalidChar(char),
    /// Input string parses to symbols but is not the canonical subtractive
    /// form for any number in `1..=3999`.
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Convert an integer `n` in `1..=3999` to its canonical Roman numeral form.
pub fn to_roman(n: u32) -> Result<String, RomanError> {
    todo!()
}

/// Parse a canonical Roman numeral string into a `u32` in `1..=3999`.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    todo!()
}
