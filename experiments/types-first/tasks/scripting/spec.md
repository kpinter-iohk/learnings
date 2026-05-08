Implement a tiny expression-language evaluator in Rust.

Grammar:

```
expr  := let | if | or
let   := "let" IDENT "=" expr "in" expr
if    := "if" expr "then" expr "else" expr
or    := and ("or" and)*
and   := not ("and" not)*
not   := "not" not | cmp
cmp   := add (("==" | "!=" | "<" | "<=" | ">" | ">=") add)?
add   := mul (("+" | "-") mul)*
mul   := unary (("*" | "/") unary)*
unary := "-" unary | atom
atom  := INT | "true" | "false" | IDENT | "(" expr ")"
```

Identifiers match `[a-z_][a-z0-9_]*` and are not any of the keywords (`let`, `in`, `if`, `then`, `else`, `or`, `and`, `not`, `true`, `false`). Whitespace separates tokens. No comments.

Values are `i64` or `bool`. There are no implicit conversions: `1 + true` is a runtime error; `if 1 then a else b` is a runtime error.

Public API:

- `pub fn parse(input: &str) -> Result<Expr, ParseError>`
- `pub fn eval(expr: &Expr) -> Result<Value, EvalError>`

The types `Expr`, `Value`, `ParseError`, and `EvalError` must be public.

Additional requirements on those types:

- `Value` must implement `std::fmt::Display` such that integer values print as decimal (e.g., `42`, `-3`) and booleans print as `true` or `false`.
- `ParseError` and `EvalError` must implement `std::fmt::Display` and `std::fmt::Debug`.
- `ParseError` must carry the byte offset within the input where parsing failed.
- `EvalError` must distinguish at least three cases: type errors, division by zero, and unbound identifiers. The exact representation is your choice.

Standard library only. The complete program is one `lib.rs`.
