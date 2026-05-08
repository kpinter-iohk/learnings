use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    OutOfRange(u32),
    Empty,
    InvalidChar(char),
    NonCanonical,
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::OutOfRange(n) => {
                write!(f, "value {} out of range; must be in 1..=3999", n)
            }
            RomanError::Empty => write!(f, "empty input"),
            RomanError::InvalidChar(c) => write!(f, "invalid character: {:?}", c),
            RomanError::NonCanonical => {
                write!(f, "input is not the canonical subtractive Roman numeral")
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

pub fn to_roman(n: u32) -> Result<String, RomanError> {
    if n == 0 || n > 3999 {
        return Err(RomanError::OutOfRange(n));
    }
    let mut remaining = n;
    let mut out = String::new();
    for &(value, sym) in PAIRS.iter() {
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
        match c {
            'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M' => {}
            other => return Err(RomanError::InvalidChar(other)),
        }
    }
    let bytes = s.as_bytes();
    let mut total: u32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        let cur = value_of(bytes[i]);
        let next = bytes.get(i + 1).map(|b| value_of(*b));
        match next {
            Some(nv) if nv > cur => {
                total += nv - cur;
                i += 2;
            }
            _ => {
                total += cur;
                i += 1;
            }
        }
        if total > 3999 {
            return Err(RomanError::NonCanonical);
        }
    }
    if total == 0 || total > 3999 {
        return Err(RomanError::NonCanonical);
    }
    let canonical = to_roman(total).map_err(|_| RomanError::NonCanonical)?;
    if canonical != s {
        return Err(RomanError::NonCanonical);
    }
    Ok(total)
}

fn value_of(b: u8) -> u32 {
    match b {
        b'I' => 1,
        b'V' => 5,
        b'X' => 10,
        b'L' => 50,
        b'C' => 100,
        b'D' => 500,
        b'M' => 1000,
        _ => unreachable!(),
    }
}
