use solution::distance;

#[test]
fn empty_to_empty_is_zero() {
    assert_eq!(distance("", ""), 0);
}

#[test]
fn empty_to_nonempty_is_length() {
    assert_eq!(distance("", "abc"), 3);
}

#[test]
fn nonempty_to_empty_is_length() {
    assert_eq!(distance("abc", ""), 3);
}

#[test]
fn identical_is_zero() {
    assert_eq!(distance("hello", "hello"), 0);
}

#[test]
fn single_substitution() {
    assert_eq!(distance("a", "b"), 1);
}

#[test]
fn single_insertion() {
    assert_eq!(distance("ab", "abc"), 1);
}

#[test]
fn single_deletion() {
    assert_eq!(distance("abc", "ab"), 1);
}

#[test]
fn classic_kitten_sitting() {
    assert_eq!(distance("kitten", "sitting"), 3);
}

#[test]
fn classic_flaw_lawn() {
    assert_eq!(distance("flaw", "lawn"), 2);
}

#[test]
fn classic_intention_execution() {
    assert_eq!(distance("intention", "execution"), 5);
}

#[test]
fn symmetric() {
    assert_eq!(distance("abcde", "axcye"), distance("axcye", "abcde"));
    assert_eq!(distance("rust", "dust"), distance("dust", "rust"));
}

#[test]
fn unicode_single_char_substitution() {
    assert_eq!(distance("é", "e"), 1);
}

#[test]
fn unicode_in_middle() {
    assert_eq!(distance("café", "cafe"), 1);
}

#[test]
fn unicode_emoji() {
    // each emoji is one Unicode code point (or grapheme); should be one char
    assert_eq!(distance("🦀", "🐍"), 1);
}

#[test]
fn transposition_counts_as_two() {
    // "ab" -> "ba" requires 2 substitutions in plain Levenshtein
    assert_eq!(distance("ab", "ba"), 2);
}

#[test]
fn prefix_extension() {
    assert_eq!(distance("abcde", "abcdefgh"), 3);
}
