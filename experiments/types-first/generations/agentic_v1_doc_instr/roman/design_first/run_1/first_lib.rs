//! Roman numeral converter for the canonical subtractive form
//! of integers in the range `1..=3999`.

use std::fmt;

/// Error type returned by [`to_roman`] and [`from_roman`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    /// The integer was 0 or greater than 3999.
    OutOfRange(u32),
    /// The input string was empty.
    Empty,
    /// The input contained a character outside `I V X L C D M`.
    InvalidChar(char),
    /// The input was syntactically a sequence of allowed symbols, but
    /// not the canonical subtractive representation of any value
    /// in `1..=3999` (e.g. `IIII`, `VV`, `IC`).
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for RomanError {}

/// Convert an integer in `1..=3999` to its canonical Roman numeral
/// representation using subtractive notation.
pub fn to_roman(n: u32) -> Result<String, RomanError> {
    todo!()
}

/// Parse a Roman numeral string into the integer it represents.
///
/// Only the canonical subtractive form is accepted; non-canonical
/// forms such as `IIII` or `VV` are rejected.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    todo!()
}
