//! Roman numeral converter.
//!
//! Public API:
//!   - `to_roman(n: u32) -> Result<String, RomanError>`
//!   - `from_roman(s: &str) -> Result<u32, RomanError>`
//!
//! Valid range is `1..=3999`. The subtractive forms `IV`, `IX`, `XL`, `XC`,
//! `CD`, `CM` are required; only uppercase `I V X L C D M` are accepted.
//! `from_roman` only accepts the canonical subtractive form, i.e. the exact
//! string `to_roman` would produce for the same number.

use std::fmt;

/// Errors returned by the Roman-numeral conversion functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    /// Numeric value is outside the supported range `1..=3999`.
    OutOfRange(u32),
    /// Input string was empty.
    Empty,
    /// Input contained a character that is not one of `I V X L C D M`.
    InvalidCharacter(char),
    /// Input is not the canonical subtractive form of any number in `1..=3999`.
    /// Examples: `IIII`, `VV`, `IC`, lowercase input.
    NotCanonical(String),
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "value {} is out of range (must be 1..=3999)", n)
            }
            RomanError::Empty => write!(f, "input string is empty"),
            RomanError::InvalidCharacter(c) => {
                write!(f, "invalid character {:?}; allowed: I V X L C D M", c)
            }
            RomanError::NotCanonical(s) => {
                write!(f, "{:?} is not a canonical Roman numeral", s)
            }
        }
    }
}

/// Pairs (value, symbol) ordered from largest to smallest, including the six
/// subtractive combinations. The greedy algorithm over this list always
/// produces the canonical Roman numeral.
const PAIRS: &[(u32, &str)] = &[
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

/// Convert a number in `1..=3999` to its canonical Roman numeral string.
pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange(n));
    }
    let mut out = String::new();
    let mut remaining = n;
    for &(value, symbol) in PAIRS {
        while remaining >= value {
            out.push_str(symbol);
            remaining -= value;
        }
    }
    Ok(out)
}

/// Numeric value of a single allowed Roman digit, or `None` if not allowed.
fn digit_value(c: char) -> Option<u32> {
    match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

/// Parse a canonical Roman numeral string into its numeric value.
///
/// The string is accepted only if it is the exact canonical subtractive form
/// produced by `to_roman` for some `n` in `1..=3999`.
pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    if s.is_empty() {
        return Err(RomanError::Empty);
    }

    // Reject any disallowed character (covers lowercase input, whitespace,
    // punctuation, non-ASCII, etc.).
    for c in s.chars() {
        if digit_value(c).is_none() {
            return Err(RomanError::InvalidCharacter(c));
        }
    }

    // Standard subtractive parse: if a digit is smaller than the next digit,
    // it counts as negative; otherwise it counts as positive. This yields
    // the value of *any* Roman-like sequence; we'll check canonicity below.
    let chars: Vec<char> = s.chars().collect();
    let mut total: u32 = 0;
    let mut i = 0;
    while i < chars.len() {
        let v = digit_value(chars[i]).expect("validated above");
        let add = if i + 1 < chars.len() {
            let nv = digit_value(chars[i + 1]).expect("validated above");
            if v < nv {
                i += 2;
                nv - v
            } else {
                i += 1;
                v
            }
        } else {
            i += 1;
            v
        };
        total = match total.checked_add(add) {
            Some(t) if t <= 3999 => t,
            _ => return Err(RomanError::NotCanonical(s.to_string())),
        };
    }

    if total == 0 {
        return Err(RomanError::NotCanonical(s.to_string()));
    }

    // Canonicity check: the only accepted spelling of `total` is the one
    // `to_roman` produces. This rejects `IIII`, `VV`, `IC`, `XCC`, `IIV`,
    // `VX`, etc., all in one shot.
    let canonical = to_roman(total).map_err(|_| RomanError::NotCanonical(s.to_string()))?;
    if canonical != s {
        return Err(RomanError::NotCanonical(s.to_string()));
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_roman_known_values() {
        assert_eq!(to_roman(1).unwrap(), "I");
        assert_eq!(to_roman(4).unwrap(), "IV");
        assert_eq!(to_roman(9).unwrap(), "IX");
        assert_eq!(to_roman(40).unwrap(), "XL");
        assert_eq!(to_roman(90).unwrap(), "XC");
        assert_eq!(to_roman(400).unwrap(), "CD");
        assert_eq!(to_roman(900).unwrap(), "CM");
        assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
        assert_eq!(to_roman(3999).unwrap(), "MMMCMXCIX");
    }

    #[test]
    fn to_roman_out_of_range() {
        assert_eq!(to_roman(0), Err(RomanError::OutOfRange(0)));
        assert_eq!(to_roman(4000), Err(RomanError::OutOfRange(4000)));
    }

    #[test]
    fn from_roman_known_values() {
        assert_eq!(from_roman("I").unwrap(), 1);
        assert_eq!(from_roman("MCMXCIV").unwrap(), 1994);
        assert_eq!(from_roman("MMMCMXCIX").unwrap(), 3999);
    }

    #[test]
    fn from_roman_rejects_bad_inputs() {
        assert_eq!(from_roman(""), Err(RomanError::Empty));
        assert!(matches!(from_roman("iv"), Err(RomanError::InvalidCharacter('i'))));
        assert!(matches!(from_roman("X1"), Err(RomanError::InvalidCharacter('1'))));
        assert!(matches!(from_roman("IIII"), Err(RomanError::NotCanonical(_))));
        assert!(matches!(from_roman("VV"),   Err(RomanError::NotCanonical(_))));
        assert!(matches!(from_roman("IC"),   Err(RomanError::NotCanonical(_))));
        assert!(matches!(from_roman("XCC"),  Err(RomanError::NotCanonical(_))));
        assert!(matches!(from_roman("IIV"),  Err(RomanError::NotCanonical(_))));
    }

    #[test]
    fn round_trip_full_range() {
        for n in 1..=3999u32 {
            let s = to_roman(n).expect("to_roman in range");
            assert_eq!(from_roman(&s), Ok(n), "round trip failed for n={}, s={}", n, s);
        }
    }

    #[test]
    fn display_error_is_non_empty() {
        let e = RomanError::OutOfRange(4000);
        assert!(!format!("{}", e).is_empty());
        assert!(!format!("{:?}", e).is_empty());
    }
}
