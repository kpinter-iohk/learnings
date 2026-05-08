Build a Roman numeral converter in Rust.

Public API:

- `pub fn to_roman(n: u32) -> Result<String, RomanError>`
- `pub fn from_roman(s: &str) -> Result<u32, RomanError>`

Valid range: `1..=3999`. Use subtractive notation: `IV`, `IX`, `XL`, `XC`, `CD`, `CM`. Allowed symbols: `I`, `V`, `X`, `L`, `C`, `D`, `M` (uppercase only).

Errors:

- `to_roman` returns `Err` if `n == 0` or `n > 3999`.
- `from_roman` returns `Err` for: an empty string; any character other than the seven allowed symbols; or any string that is not the canonical subtractive form for some `n` in `1..=3999` (so `IIII`, `VV`, `IC`, lowercase input — all invalid).

Round-trip property: for every `n` in `1..=3999`, `from_roman(&to_roman(n)?) == Ok(n)`.

`RomanError` must be a public type implementing `Debug` and `Display`. Its internal shape is your choice.

Standard library only. The complete program is one `lib.rs`.
