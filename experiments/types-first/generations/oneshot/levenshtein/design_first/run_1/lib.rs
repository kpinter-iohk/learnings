pub fn distance(a: &str, b: &str) -> usize {
    // Collect into chars so we operate on Unicode code points, not bytes.
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let m = a_chars.len();
    let n = b_chars.len();

    // Trivial cases.
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    // Allocate two rows of length n + 1.
    let mut rows = DpRows::new(n);
    rows.init_baseline();

    for i in 1..=m {
        // First column: cost of deleting `i` characters from `a` to reach "".
        rows.curr[0] = i;

        for j in 1..=n {
            let same = a_chars[i - 1] == b_chars[j - 1];

            let sub_cost = if same { 0 } else { cost_of(Edit::Substitute) };
            let del_cost = cost_of(Edit::Delete);
            let ins_cost = cost_of(Edit::Insert);

            let substitute = rows.prev[j - 1] + sub_cost;
            let delete = rows.prev[j] + del_cost;
            let insert = rows.curr[j - 1] + ins_cost;

            rows.curr[j] = min3(substitute, delete, insert);
        }

        rows.rotate();
    }

    // After the final rotate, the answer sits in `prev`.
    rows.prev[n]
}

/// Minimum of three `usize` values.
fn min3(x: usize, y: usize, z: usize) -> usize {
    let xy = if x < y { x } else { y };
    if xy < z { xy } else { z }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        assert_eq!(distance("", ""), 0);
        assert_eq!(distance("a", "a"), 0);
        assert_eq!(distance("kitten", "kitten"), 0);
        assert_eq!(distance("héllo", "héllo"), 0);
    }

    #[test]
    fn empty_baseline() {
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("", "héllo"), "héllo".chars().count());
        assert_eq!(distance("", "🦀🦀🦀"), 3);
    }

    #[test]
    fn classic_examples() {
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("flaw", "lawn"), 2);
        assert_eq!(distance("gumbo", "gambol"), 2);
        assert_eq!(distance("book", "back"), 2);
    }

    #[test]
    fn unicode_code_points() {
        // "é" is multibyte in UTF-8 but 1 code point.
        assert_eq!(distance("é", "e"), 1);
        assert_eq!(distance("café", "cafe"), 1);
        assert_eq!(distance("🦀", ""), 1);
        assert_eq!(distance("🦀a", "a🦀"), 2);
    }

    #[test]
    fn symmetry() {
        let pairs = [
            ("", "abc"),
            ("kitten", "sitting"),
            ("café", "cafe"),
            ("🦀a", "a🦀"),
            ("abcdef", "azced"),
        ];
        for (a, b) in pairs {
            assert_eq!(distance(a, b), distance(b, a), "symmetry failed: {a:?} {b:?}");
        }
    }

    #[test]
    fn single_edits() {
        // Insertion.
        assert_eq!(distance("cat", "cats"), 1);
        // Deletion.
        assert_eq!(distance("cats", "cat"), 1);
        // Substitution.
        assert_eq!(distance("cat", "bat"), 1);
    }
}
