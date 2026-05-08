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
            RomanError::OutOfRange => {
                write!(f, "value out of range: must be between 1 and 3999")
            }
            RomanError::Empty => write!(f, "input string is empty"),
            RomanError::InvalidCharacter(c) => {
                write!(f, "invalid character in roman numeral: {:?}", c)
            }
            RomanError::NotCanonical => {
                write!(f, "string is not a canonical roman numeral")
            }
        }
    }
}

impl std::error::Error for RomanError {}

const TABLE: &[(u32, &str)] = &[
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
    for &(value, sym) in TABLE {
        while remaining >= value {
            out.push_str(sym);
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

    let mut total: u32 = 0;
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let v = symbol_value(c).ok_or(RomanError::InvalidCharacter(c))?;
        let next = chars.get(i + 1).and_then(|&nc| symbol_value(nc));
        match next {
            Some(nv) if nv > v => total = total.checked_sub(v).ok_or(RomanError::NotCanonical)
                .map(|t| t)
                .unwrap_or(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0).wrapping_add(0),
            _ => total += v,
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
