pub fn distance(a: &str, b: &str) -> usize {
    // Collect chars once so we can index them in O(1) and avoid
    // repeated UTF-8 decoding in the inner loop.
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let n = a_chars.len();
    let m = b_chars.len();

    // Trivial cases: one of the strings is empty.
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    // Ensure the shorter sequence is `b_chars` so the row vector is
    // as small as possible. Distance is symmetric, so swapping is fine.
    let (a_chars, b_chars, n, m) = if m > n {
        (b_chars, a_chars, m, n)
    } else {
        (a_chars, b_chars, n, m)
    };

    // `prev[j]` holds the distance for the previous row of the DP
    // table: dist(a_chars[..i], b_chars[..j]).
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        let ca = a_chars[i - 1];
        for j in 1..=m {
            let cb = b_chars[j - 1];
            let substitution_cost = if ca == cb { 0 } else { 1 };

            let deletion = prev[j] + 1;
            let insertion = curr[j - 1] + 1;
            let substitution = prev[j - 1] + substitution_cost;

            curr[j] = deletion.min(insertion).min(substitution);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    // After the final swap, the answer lives in `prev`.
    prev[m]
}

#[cfg(test)]
mod tests {
    use super::distance;

    #[test]
    fn identity() {
        assert_eq!(distance("", ""), 0);
        assert_eq!(distance("a", "a"), 0);
        assert_eq!(distance("hello", "hello"), 0);
        assert_eq!(distance("héllo", "héllo"), 0);
    }

    #[test]
    fn empty_baseline() {
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("", "héllo"), "héllo".chars().count());
        assert_eq!(distance("日本語", ""), 3);
    }

    #[test]
    fn symmetry() {
        let pairs = [
            ("kitten", "sitting"),
            ("flaw", "lawn"),
            ("gumbo", "gambol"),
            ("é", "e"),
            ("café", "cafe"),
            ("日本語", "本日語"),
        ];
        for (a, b) in pairs {
            assert_eq!(distance(a, b), distance(b, a), "symmetry failed for {a:?} vs {b:?}");
        }
    }

    #[test]
    fn classic_examples() {
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("flaw", "lawn"), 2);
        assert_eq!(distance("gumbo", "gambol"), 2);
        assert_eq!(distance("book", "back"), 2);
    }

    #[test]
    fn unicode_is_codepoint_based() {
        // "é" is two bytes in UTF-8 but one char.
        assert_eq!(distance("é", "e"), 1);
        assert_eq!(distance("café", "cafe"), 1);
        // A single CJK substitution.
        assert_eq!(distance("日本語", "中本語"), 1);
    }

    #[test]
    fn pure_insertions_and_deletions() {
        assert_eq!(distance("abc", "abcdef"), 3);
        assert_eq!(distance("abcdef", "abc"), 3);
    }
}
