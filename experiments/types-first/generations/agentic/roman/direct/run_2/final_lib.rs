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
            RomanError::Empty => write!(f, "empty input string"),
            RomanError::InvalidCharacter(c) => write!(f, "invalid character: {:?}", c),
            RomanError::NotCanonical => write!(f, "not a canonical Roman numeral"),
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

pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange);
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
    let mut total: i64 = 0;
    for i in 0..values.len() {
        let v = values[i] as i64;
        let next = values.get(i + 1).copied().unwrap_or(0) as i64;
        if v < next {
            total -= v;
        } else {
            total += v;
        }
    }
    if total < 1 || total > 3999 {
        return Err(RomanError::NotCanonical);
    }
    let n = total as u32;
    let canonical = to_roman(n).map_err(|_| RomanError::NotCanonical)?;
    if canonical != s {
        return Err(RomanError::NotCanonical);
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_full_range() {
        for n in 1..=3999u32 {
            let s = to_roman(n).unwrap();
            assert_eq!(from_roman(&s), Ok(n));
        }
    }

    #[test]
    fn to_roman_known() {
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
    fn to_roman_out_of_range() {
        assert!(to_roman(0).is_err());
        assert!(to_roman(4000).is_err());
    }

    #[test]
    fn from_roman_invalid() {
        assert!(from_roman("").is_err());
        assert!(from_roman("IIII").is_err());
        assert!(from_roman("VV").is_err());
        assert!(from_roman("IC").is_err());
        assert!(from_roman("iv").is_err());
        assert!(from_roman("ABC").is_err());
    }
}
