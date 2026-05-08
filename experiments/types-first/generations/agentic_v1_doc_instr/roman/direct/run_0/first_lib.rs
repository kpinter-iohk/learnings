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
            RomanError::OutOfRange => write!(f, "value out of range (1..=3999)"),
            RomanError::Empty => write!(f, "empty input"),
            RomanError::InvalidCharacter(c) => write!(f, "invalid character: {:?}", c),
            RomanError::NotCanonical => write!(f, "not canonical roman numeral form"),
        }
    }
}

impl std::error::Error for RomanError {}

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
        return Err(RomanError::OutOfRange);
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

fn char_value(c: char) -> Option<u32> {
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

pub fn from_roman(s: &str) -> Result<u32, RomanError> {
    if s.is_empty() {
        return Err(RomanError::Empty);
    }
    let chars: Vec<char> = s.chars().collect();
    let mut values = Vec::with_capacity(chars.len());
    for &c in &chars {
        match char_value(c) {
            Some(v) => values.push(v),
            None => return Err(RomanError::InvalidCharacter(c)),
        }
    }

    let mut total: u32 = 0;
    let mut i = 0;
    while i < values.len() {
        let cur = values[i];
        if i + 1 < values.len() && values[i + 1] > cur {
            total = total.checked_add(values[i + 1] - cur).ok_or(RomanError::NotCanonical)?;
            i += 2;
        } else {
            total = total.checked_add(cur).ok_or(RomanError::NotCanonical)?;
            i += 1;
        }
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
        for n in 1..=3999u32 {
            let s = to_roman(n).unwrap();
            assert_eq!(from_roman(&s), Ok(n), "failed at {}", n);
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
        assert_eq!(to_roman(40).unwrap(), "XL");
        assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
        assert_eq!(to_roman(3999).unwrap(), "MMMCMXCIX");
    }
}
