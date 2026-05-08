use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange,
    Empty,
    InvalidChar(char),
    NotCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::OutOfRange => write!(f, "value out of range (must be 1..=3999)"),
            RomanError::Empty => write!(f, "empty input"),
            RomanError::InvalidChar(c) => write!(f, "invalid character: {:?}", c),
            RomanError::NotCanonical => write!(f, "not canonical Roman numeral form"),
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
    let mut values: Vec<u32> = Vec::with_capacity(chars.len());
    for &c in &chars {
        match char_value(c) {
            Some(v) => values.push(v),
            None => return Err(RomanError::InvalidChar(c)),
        }
    }
    let mut total: u32 = 0;
    let mut i = 0;
    while i < values.len() {
        let cur = values[i];
        let next = values.get(i + 1).copied().unwrap_or(0);
        if cur < next {
            total = total.checked_add(next - cur).ok_or(RomanError::NotCanonical)?;
            i += 2;
        } else {
            total = total.checked_add(cur).ok_or(RomanError::NotCanonical)?;
            i += 1;
        }
    }
    if total == 0 || total > 3999 {
        return Err(RomanError::NotCanonical);
    }
    match to_roman(total) {
        Ok(canon) if canon == s => Ok(total),
        _ => Err(RomanError::NotCanonical),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
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
    fn oob() {
        assert!(to_roman(0).is_err());
        assert!(to_roman(4000).is_err());
    }

    #[test]
    fn parse_basics() {
        assert_eq!(from_roman("I").unwrap(), 1);
        assert_eq!(from_roman("MCMXCIV").unwrap(), 1994);
        assert_eq!(from_roman("MMMCMXCIX").unwrap(), 3999);
    }

    #[test]
    fn parse_invalid() {
        assert!(from_roman("").is_err());
        assert!(from_roman("IIII").is_err());
        assert!(from_roman("VV").is_err());
        assert!(from_roman("IC").is_err());
        assert!(from_roman("iv").is_err());
        assert!(from_roman("ABC").is_err());
    }

    #[test]
    fn round_trip() {
        for n in 1..=3999 {
            let s = to_roman(n).unwrap();
            assert_eq!(from_roman(&s).unwrap(), n);
        }
    }
}
