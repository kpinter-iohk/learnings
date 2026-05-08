pub fn distance(a: &str, b: &str) -> usize {
    // Collect once so we can index by char position in O(1) and avoid
    // repeated UTF-8 decoding inside the inner loop.
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    // Fast paths for the empty-string baselines.
    if a_chars.is_empty() {
        return b_chars.len();
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    // Use the shorter sequence as the row width to minimise memory.
    // Symmetry of Levenshtein distance makes this safe.
    let (outer, inner) = if a_chars.len() >= b_chars.len() {
        (&a_chars, &b_chars)
    } else {
        (&b_chars, &a_chars)
    };

    let mut row = DpRow::seeded(inner.len());
    for &c in outer.iter() {
        row.advance(c, inner);
    }
    row.answer()
}

// --- Internal helpers -------------------------------------------------------

/// A single-row dynamic-programming buffer.
///
/// We only need two adjacent rows of the classic edit-distance matrix at any
/// time, so we keep the cost of the previous row and overwrite it as we
/// advance. This struct exists to make that intent explicit.
struct DpRow {
    cells: Vec<usize>,
}

impl DpRow {
    /// Build the initial row corresponding to transforming the empty prefix
    /// of `a` into prefixes of `b` of lengths `0..=width` — i.e. `[0, 1, 2, ..., width]`.
    fn seeded(width: usize) -> Self {
        let mut cells = Vec::with_capacity(width + 1);
        for j in 0..=width {
            cells.push(j);
        }
        DpRow { cells }
    }

    /// Advance to the next row given the current `a`-character `ca` and the
    /// full sequence of `b`-characters `bs`. After this call, `self` holds
    /// the row for the next prefix of `a`.
    fn advance(&mut self, ca: char, bs: &[char]) {
        // `diag` tracks the value at (i-1, j-1) before it gets overwritten.
        // `self.cells[0]` corresponds to the cost of deleting all `i`
        // characters of `a` so far, which grows by 1 each row.
        let mut diag = self.cells[0];
        self.cells[0] += 1;

        for (j, &cb) in bs.iter().enumerate() {
            let above = self.cells[j + 1]; // (i-1, j) — soon to be overwritten
            let left = self.cells[j];      // (i,   j-1) — already updated

            let sub_cost = if ca == cb { 0 } else { 1 };

            let new_val = min3(
                above + 1,        // deletion from `a`
                left + 1,         // insertion into `a`
                diag + sub_cost,  // match or substitution
            );

            self.cells[j + 1] = new_val;
            diag = above;
        }
    }

    /// The final cell — i.e. the edit distance once both strings are fully
    /// consumed.
    fn answer(&self) -> usize {
        *self.cells.last().expect("DpRow always has at least one cell")
    }
}

/// Pick the smallest of three `usize` costs.
fn min3(x: usize, y: usize, z: usize) -> usize {
    let xy = if x < y { x } else { y };
    if xy < z { xy } else { z }
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::distance;

    #[test]
    fn identity() {
        for s in ["", "a", "hello", "café", "🦀🦀🦀"] {
            assert_eq!(distance(s, s), 0, "identity failed for {:?}", s);
        }
    }

    #[test]
    fn symmetry() {
        let pairs = [
            ("", "abc"),
            ("kitten", "sitting"),
            ("flaw", "lawn"),
            ("café", "cafe"),
            ("é", "e"),
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
        for s in ["", "a", "abc", "café", "🦀🦀🦀"] {
            let n = s.chars().count();
            assert_eq!(distance("", s), n);
            assert_eq!(distance(s, ""), n);
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
        // Crab emoji is 4 bytes but one char.
        assert_eq!(distance("🦀", ""), 1);
        assert_eq!(distance("🦀a", "a🦀"), 2);
    }

    #[test]
    fn single_edits() {
        assert_eq!(distance("abc", "abcd"), 1); // insertion
        assert_eq!(distance("abcd", "abc"), 1); // deletion
        assert_eq!(distance("abc", "abd"), 1); // substitution
    }
}
