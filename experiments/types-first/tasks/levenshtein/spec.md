Build a Levenshtein distance function in Rust.

Public API:

- `pub fn distance(a: &str, b: &str) -> usize`

Returns the minimum number of single-character insertions, deletions, or substitutions required to transform `a` into `b`.

Operates on Unicode code points (`char`s), not bytes. So `distance("é", "e") == 1` even though "é" is multiple bytes in UTF-8.

Properties (must hold for all inputs):
- `distance(a, a) == 0` (identity)
- `distance(a, b) == distance(b, a)` (symmetry)
- `distance("", a) == a.chars().count()` (insertion-only baseline)

Standard library only. The complete program is one `lib.rs`.
