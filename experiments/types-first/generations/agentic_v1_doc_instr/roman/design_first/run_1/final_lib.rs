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
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "value {} is out of range 1..=3999", n)
            }
            RomanError::Empty => write!(f, "input string is empty"),
            RomanError::InvalidChar(c) => {
                write!(f, "invalid Roman numeral character: {:?}", c)
            }
            RomanError::NotCanonical => {
                write!(f, "input is not a canonical Roman numeral")
            }
        }
    }
}

impl std::error::Error for RomanError {}

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

/// Convert an integer in `1..=3999` to its canonical Roman numeral
/// representation using subtractive notation.
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

/// Parse a Roman numeral string into the integer it represents.
///
/// Only the canonical subtractive form is accepted; non-canonical
/// forms such as `IIII` or `VV` are rejected.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    if s.is_empty() {
        return Err(RomanError::Empty);
    }
    for c in s.chars() {
        match c {
            'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M' => {}
            other => return Err(RomanError::InvalidChar(other)),
        }
    }
    let mut value: u32 = 0;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let cur = digit_value(bytes[i]);
        let next = bytes.get(i + 1).map(|&b| digit_value(b));
        match next {
            Some(n) if n > cur => {
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
    // Canonicalization check: re-encode and compare.
    let canonical = to_roman(value).map_err(|_| RomanError::NotCanonical)?;
    if canonical != s {
        return Err(RomanError::NotCanonical);
    }
    Ok(value)
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
        _ => unreachable!("validated by from_roman"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        assert_eq!(to_roman(1).unwrap(), "I");
        assert_eq!(to_roman(4).unwrap(), "IV");
        assert_eq!(to_roman(9).unwrap(), "IX");
        assert_eq!(to_roman(40).unwrap(), "XL");
        assert_eq!(to_roman(90).unwrap(), "XC");
        assert_eq!(to_roman(400).unwrap(), "CD");
        assert_eq!(to_roman(900).unwrap(), "CM");
        assert_eq!(to_roman(3999).unwrap(), "MMMCMXCIX");
        assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
    }

    #[test]
    fn out_of_range() {
        assert!(matches!(to_roman(0), Err(RomanError::OutOfRange(0))));
        assert!(matches!(to_roman(4000), Err(RomanError::OutOfRange(4000))));
    }

    #[test]
    fn rejects_non_canonical() {
        assert!(matches!(from_roman(""), Err(RomanError::Empty)));
        assert!(matches!(from_roman("iv"), Err(RomanError::InvalidChar('i'))));
        assert!(matches!(from_roman("IIII"), Err(RomanError::NotCanonical)));
        assert!(matches!(from_roman("VV"), Err(RomanError::NotCanonical)));
        assert!(matches!(from_roman("IC"), Err(RomanError::NotCanonical)));
    }

    #[test]
    fn round_trip() {
        for n in 1..=3999u32 {
            let s = to_roman(n).unwrap();
            assert_eq!(from_roman(&s), Ok(n));
        }
    }
}
