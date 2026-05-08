pub fn distance(a: &str, b: &str) -> usize {
    // Decode once into char vectors so indexing is O(1) and the DP loop
    // doesn't pay UTF-8 decoding costs on every access.
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    distance_chars(&av, &bv)
}

// --- Internal helpers -------------------------------------------------------

/// Minimum of three `usize` values. Used for the DP recurrence:
/// `min(delete, insert, substitute)`.
fn min3(x: usize, y: usize, z: usize) -> usize {
    let xy = if x < y { x } else { y };
    if xy < z { xy } else { z }
}

/// Core DP routine operating on already-decoded `char` slices.
///
/// Uses the standard two-row (here: single-row + scalar) space optimization,
/// reducing memory from O(|a| * |b|) to O(min(|a|, |b|)).
fn distance_chars(a: &[char], b: &[char]) -> usize {
    // Fast paths for empty inputs — also satisfy the `distance("", x)` baseline.
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    // Make `b` the shorter side so the working row is as small as possible.
    // Levenshtein distance is symmetric, so this swap is sound.
    let (a, b) = if a.len() < b.len() { (b, a) } else { (a, b) };

    let n = a.len();
    let m = b.len();

    // `prev[j]` holds the edit distance between a[..i] and b[..j] for the
    // previous row `i-1`. We seed with row 0: dist("", b[..j]) == j.
    let mut prev: Vec<usize> = (0..=m).collect();

    for i in 1..=n {
        // First column: dist(a[..i], "") == i.
        let mut prev_diag = prev[0];
        prev[0] = i;

        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };

            let deletion = prev[j] + 1;          // drop a[i-1]
            let insertion = prev[j - 1] + 1;     // insert b[j-1]
            let substitution = prev_diag + cost; // align a[i-1] with b[j-1]

            // Save the value that will become `prev_diag` for the next column
            // *before* we overwrite `prev[j]`.
            let next_diag = prev[j];
            prev[j] = min3(deletion, insertion, substitution);
            prev_diag = next_diag;
        }
    }

    prev[m]
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        for s in &["", "a", "kitten", "naïveté", "🦀🦀🦀", "hello world"] {
            assert_eq!(distance(s, s), 0, "identity failed for {:?}", s);
        }
    }

    #[test]
    fn symmetry() {
        let pairs = [
            ("", "abc"),
            ("kitten", "sitting"),
            ("flaw", "lawn"),
            ("é", "e"),
            ("naïveté", "naivete"),
            ("🦀rust", "rust🦀"),
        ];
        for (a, b) in pairs {
            assert_eq!(
                distance(a, b),
                distance(b, a),
                "symmetry failed for ({:?}, {:?})",
                a,
                b
            );
        }
    }

    #[test]
    fn empty_baseline() {
        for s in &["", "a", "abc", "naïveté", "🦀🦀🦀"] {
            let n = s.chars().count();
            assert_eq!(distance("", s), n);
            assert_eq!(distance(s, ""), n);
        }
    }

    #[test]
    fn unicode_code_points() {
        // "é" is 2 bytes in UTF-8 but a single char; distance to "e" must be 1.
        assert_eq!(distance("é", "e"), 1);
        assert_eq!(distance("naïveté", "naivete"), 2);
        // Crab emoji is 4 bytes but counts as one code point.
        assert_eq!(distance("🦀", ""), 1);
        assert_eq!(distance("🦀a", "a🦀"), 2);
    }

    #[test]
    fn classic_examples() {
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("flaw", "lawn"), 2);
        assert_eq!(distance("gumbo", "gambol"), 2);
        assert_eq!(distance("book", "back"), 2);
        assert_eq!(distance("abc", "abc"), 0);
        assert_eq!(distance("abc", "abd"), 1);
        assert_eq!(distance("abc", ""), 3);
    }

    #[test]
    fn triangle_inequality_spot_check() {
        // d(a, c) <= d(a, b) + d(b, c) — sanity check, not exhaustive.
        let triples = [
            ("kitten", "sitting", "smitten"),
            ("flaw", "lawn", "law"),
            ("", "abc", "abcd"),
        ];
        for (a, b, c) in triples {
            assert!(distance(a, c) <= distance(a, b) + distance(b, c));
        }
    }
}
