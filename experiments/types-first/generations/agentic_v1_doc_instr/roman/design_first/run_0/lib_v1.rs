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
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "value {} is out of range 1..=3999", n)
            }
            RomanError::Empty => f.write_str("input string is empty"),
            RomanError::InvalidChar(c) => {
                write!(f, "invalid Roman numeral character: {:?}", c)
            }
            RomanError::NotCanonical => {
                f.write_str("input is not a canonical Roman numeral")
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

/// Convert an integer `n` in `1..=3999` to its canonical Roman numeral form.
pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange(n));
    }
    let mut out = String::new();
    let mut rem = n;
    for &(val, sym) in &PAIRS {
        while rem >= val {
            out.push_str(sym);
            rem -= val;
        }
    }
    Ok(out)
}

/// Parse a canonical Roman numeral string into a `u32` in `1..=3999`.
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
    for &(val, sym) in &PAIRS {
        let sb = sym.as_bytes();
        while i + sb.len() <= bytes.len() && &bytes[i..i + sb.len()] == sb {
            value = value.saturating_add(val);
            i += sb.len();
            if value > 3999 {
                return Err(RomanError::NotCanonical);
            }
        }
    }
    if i != bytes.len() || value == 0 || value > 3999 {
        return Err(RomanError::NotCanonical);
    }
    if to_roman(value).as_deref() != Ok(s) {
        return Err(RomanError::NotCanonical);
    }
    Ok(value)
}
