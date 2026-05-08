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
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "value {} is out of range 1..=3999", n)
            }
            RomanError::Empty => write!(f, "input is empty"),
            RomanError::InvalidChar(c) => {
                write!(f, "invalid character {:?} (allowed: I V X L C D M)", c)
            }
            RomanError::NotCanonical => {
                write!(f, "input is not a canonical Roman numeral in 1..=3999")
            }
        }
    }
}

const PAIRS: [(u32, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

/// Convert an integer in `1..=3999` to its canonical Roman numeral representation.
pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange(n));
    }
    let mut remaining = n;
    let mut out = String::new();
    for &(value, sym) in &PAIRS {
        while remaining >= value {
            out.push_str(sym);
            remaining -= value;
        }
    }
    Ok(out)
}

/// Parse a canonical Roman numeral in `1..=3999` back to an integer.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    if s.is_empty() {
        return Err(RomanError::Empty);
    }
    for c in s.chars() {
        if !matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M') {
            return Err(RomanError::InvalidChar(c));
        }
    }

    let bytes = s.as_bytes();
    let mut value: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        let cur = digit_value(bytes[i]);
        let next = bytes.get(i + 1).copied().map(digit_value);
        match next {
            Some(n) if cur < n => {
                value += n - cur;
                i += 2;
            }
            _ => {
                value += cur;
                i += 1;
            }
        }
    }

    if value == 0 || value > 3999 {
        return Err(RomanError::NotCanonical);
    }

    let canonical = to_roman(value).map_err(|_| RomanError::NotCanonical)?;
    if canonical == s {
        Ok(value)
    } else {
        Err(RomanError::NotCanonical)
    }
}

fn digit_value(b: u8) -> u32 {
    match b {
        b'I' => 1,
        b'V' => 5,
        b'X' => 10,
        b'L' => 50,
        b'C' => 100,
        b'D' => 500,
        b'M' => 1000,
        _ => unreachable!(),
    }
}
