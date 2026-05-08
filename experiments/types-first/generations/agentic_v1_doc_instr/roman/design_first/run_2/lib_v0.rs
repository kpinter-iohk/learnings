//! Roman numeral converter.
//!
//! Supports values in `1..=3999` using canonical subtractive notation.

use std::fmt;

/// Error returned by [`to_roman`] and [`from_roman`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    /// The integer is outside the supported range `1..=3999`.
    OutOfRange(u32),
    /// The input string is empty.
    Empty,
    /// The input contains a character that is not one of `I V X L C D M`.
    InvalidChar(char),
    /// The input parses but is not the canonical subtractive form
    /// (e.g. `IIII`, `VV`, `IC`, or anything whose value falls outside `1..=3999`).
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Convert an integer in `1..=3999` to its canonical Roman numeral representation.
pub fn to_roman(n: u32) -> Result<String, RomanError> {
    todo!()
}

/// Parse a canonical Roman numeral in `1..=3999` back to an integer.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    todo!()
}
