use solution::{from_roman, to_roman};

#[test]
fn to_one() {
    assert_eq!(to_roman(1).unwrap(), "I");
}

#[test]
fn to_four() {
    assert_eq!(to_roman(4).unwrap(), "IV");
}

#[test]
fn to_nine() {
    assert_eq!(to_roman(9).unwrap(), "IX");
}

#[test]
fn to_forty() {
    assert_eq!(to_roman(40).unwrap(), "XL");
}

#[test]
fn to_ninety() {
    assert_eq!(to_roman(90).unwrap(), "XC");
}

#[test]
fn to_four_hundred() {
    assert_eq!(to_roman(400).unwrap(), "CD");
}

#[test]
fn to_nine_hundred() {
    assert_eq!(to_roman(900).unwrap(), "CM");
}

#[test]
fn to_three_thousand_nine_hundred_ninety_nine() {
    assert_eq!(to_roman(3999).unwrap(), "MMMCMXCIX");
}

#[test]
fn to_zero_is_error() {
    assert!(to_roman(0).is_err());
}

#[test]
fn to_four_thousand_is_error() {
    assert!(to_roman(4000).is_err());
}

#[test]
fn from_one() {
    assert_eq!(from_roman("I").unwrap(), 1);
}

#[test]
fn from_four() {
    assert_eq!(from_roman("IV").unwrap(), 4);
}

#[test]
fn from_three_thousand_nine_hundred_ninety_nine() {
    assert_eq!(from_roman("MMMCMXCIX").unwrap(), 3999);
}

#[test]
fn from_empty_is_error() {
    assert!(from_roman("").is_err());
}

#[test]
fn from_iiii_is_error() {
    assert!(from_roman("IIII").is_err());
}

#[test]
fn from_vv_is_error() {
    assert!(from_roman("VV").is_err());
}

#[test]
fn from_ic_is_error() {
    assert!(from_roman("IC").is_err());
}

#[test]
fn from_lowercase_is_error() {
    assert!(from_roman("iv").is_err());
}

#[test]
fn from_invalid_chars_is_error() {
    assert!(from_roman("ABC").is_err());
}

#[test]
fn round_trip_full_range() {
    for n in 1..=3999u32 {
        let s = to_roman(n).unwrap_or_else(|_| panic!("to_roman failed for {n}"));
        let back = from_roman(&s).unwrap_or_else(|_| panic!("from_roman failed for {s} (n={n})"));
        assert_eq!(back, n, "round-trip mismatch for {n}: got {s} -> {back}");
    }
}
