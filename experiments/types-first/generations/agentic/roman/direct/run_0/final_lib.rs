use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange,
    Empty,
    InvalidCharacter(char),
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::OutOfRange => write!(f, "value out of range (must be 1..=3999)"),
            RomanError::Empty => write!(f, "empty string"),
            RomanError::InvalidCharacter(c) => write!(f, "invalid character: {:?}", c),
            RomanError::NotCanonical => write!(f, "not a canonical Roman numeral"),
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

pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange);
    }
    let mut remaining = n;
    let mut out = String::new();
    for (value, symbol) in PAIRS.iter() {
        while remaining >= *value {
            out.push_str(symbol);
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
        match c {
            'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M' => {}
            other => return Err(RomanError::InvalidCharacter(other)),
        }
    }

    let mut remaining = s;
    let mut total: u32 = 0;
    'outer: while !remaining.is_empty() {
        for (value, symbol) in PAIRS.iter() {
            if let Some(rest) = remaining.strip_prefix(symbol) {
                total = total.checked_add(*value).ok_or(RomanError::NotCanonical)?;
                if total > 3999 {
                    return Err(RomanError::NotCanonical);
                }
                remaining = rest;
                continue 'outer;
            }
        }
        return Err(RomanError::NotCanonical);
    }

    if total == 0 || total > 3999 {
        return Err(RomanError::NotCanonical);
    }

    let canonical = to_roman(total).map_err(|_| RomanError::NotCanonical)?;
    if canonical != s {
        return Err(RomanError::NotCanonical);
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_all() {
        for n in 1..=3999 {
            let r = to_roman(n).unwrap();
            assert_eq!(from_roman(&r), Ok(n));
        }
    }

    #[test]
    fn out_of_range() {
        assert!(to_roman(0).is_err());
        assert!(to_roman(4000).is_err());
    }

    #[test]
    fn invalid_inputs() {
        assert!(from_roman("").is_err());
        assert!(from_roman("IIII").is_err());
        assert!(from_roman("VV").is_err());
        assert!(from_roman("IC").is_err());
        assert!(from_roman("iv").is_err());
        assert!(from_roman("ABC").is_err());
    }

    #[test]
    fn known_values() {
        assert_eq!(to_roman(1).unwrap(), "I");
        assert_eq!(to_roman(4).unwrap(), "IV");
        assert_eq!(to_roman(9).unwrap(), "IX");
        assert_eq!(to_roman(58).unwrap(), "LVIII");
        assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
        assert_eq!(to_roman(3999).unwrap(), "MMMCMXCIX");
    }
}
