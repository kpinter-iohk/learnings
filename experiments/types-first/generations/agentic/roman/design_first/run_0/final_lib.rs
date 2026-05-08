use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange(u32),
    Empty,
    InvalidChar(char),
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "value {} is out of range (must be 1..=3999)", n)
            }
            RomanError::Empty => write!(f, "input string is empty"),
            RomanError::InvalidChar(c) => write!(f, "invalid character '{}'", c),
            RomanError::NotCanonical => {
                write!(f, "input is not a canonical Roman numeral")
            }
        }
    }
}

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

pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange(n));
    }
    let mut remaining = n;
    let mut out = String::new();
    for &(value, sym) in PAIRS {
        while remaining >= value {
            out.push_str(sym);
            remaining -= value;
        }
    }
    Ok(out)
}

pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    if s.is_empty() {
        return Err(RomanError::Empty);
    }
    for c in s.chars() {
        if !matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M') {
            return Err(RomanError::InvalidChar(c));
        }
    }
    let mut total: u32 = 0;
    let mut rest = s;
    for &(value, sym) in PAIRS {
        while let Some(stripped) = rest.strip_prefix(sym) {
            total += value;
            rest = stripped;
            if total > 3999 {
                return Err(RomanError::NotCanonical);
            }
        }
    }
    if !rest.is_empty() || total == 0 {
        return Err(RomanError::NotCanonical);
    }
    match to_roman(total) {
        Ok(canonical) if canonical == s => Ok(total),
        _ => Err(RomanError::NotCanonical),
    }
}
