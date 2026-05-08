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
    for &(value, symbol) in PAIRS {
        while remaining >= value {
            out.push_str(symbol);
            remaining -= value;
        }
    }
    Ok(out)
}

fn symbol_value(c: char) -> Option<u32> {
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
    let mut total: u32 = 0;
    for (i, &c) in chars.iter().enumerate() {
        let v = symbol_value(c).ok_or(RomanError::InvalidCharacter(c))?;
        let next_v = chars.get(i + 1).and_then(|&nc| symbol_value(nc));
        match next_v {
            Some(nv) if nv > v => total = total.wrapping_sub(v),
            _ => total = total.wrapping_add(v),
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
    fn basic_to_roman() {
        assert_eq!(to_roman(1).unwrap(), "I");
        assert_eq!(to_roman(4).unwrap(), "IV");
        assert_eq!(to_roman(9).unwrap(), "IX");
        assert_eq!(to_roman(58).unwrap(), "LVIII");
        assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
        assert_eq!(to_roman(3999).unwrap(), "MMMCMXCIX");
    }

    #[test]
    fn out_of_range() {
        assert!(to_roman(0).is_err());
        assert!(to_roman(4000).is_err());
    }

    #[test]
    fn basic_from_roman() {
        assert_eq!(from_roman("I").unwrap(), 1);
        assert_eq!(from_roman("IV").unwrap(), 4);
        assert_eq!(from_roman("MCMXCIV").unwrap(), 1994);
    }

    #[test]
    fn invalid_from_roman() {
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
