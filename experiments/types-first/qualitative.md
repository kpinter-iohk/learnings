# Qualitative comparison — direct vs design-first

Three pairs, one per task, agentic mode, `run_0` of each treatment. My reading, knowing the labels (so this is exploratory, not blinded).

## Roman numeral converter

**Direct:** 142 LOC including 36 lines of inline `#[cfg(test)]` tests the spec didn't ask for. Implementation proper is ~106 LOC.
**Design-first:** 96 LOC, no inline tests.

When you strip out direct's volunteered inline tests, the implementations are similar size — but the type design diverges:

| | Direct | Design-first |
|---|---|---|
| `RomanError::OutOfRange` | unit variant | `OutOfRange(u32)` — carries the offending value |
| Doc comments | None | Every public item, including variants |
| `impl std::error::Error` | Yes (volunteered, not in spec) | No |
| Char validation | Helper fn `char_value(c) -> Option<u32>` + match | Inline `matches!(c, 'I' \| 'V' \| ...)` |
| `from_roman` strategy | Tokenize to `Vec<u32>`, then walk with subtractive look-ahead | Greedy walk through the `(value, symbol)` pairs table directly |

Direct's `OutOfRange` throws away the offending value; design-first keeps it. Design-first's parser is ~5 lines shorter and arguably more elegant (no intermediate `Vec` allocation).

Net: the Roman delta is mostly **doc comments** and **error-variant payload richness**.

## Job queue with retries

**Direct:** 165 LOC, no doc comments on any public item.
**Design-first:** 171 LOC, doc comment on every public item.

Same overall shape (HashMap-backed, monotonic IDs). Differences are in defensive coding and idiom:

| | Direct | Design-first |
|---|---|---|
| `JobState` derives | `Debug, PartialEq, Eq` | `Debug, PartialEq, Eq, Clone, Copy` |
| `get_state` | Manual `match` reconstructing every variant by name | One-liner: `.map(\|j\| j.state)` (works because of `Copy`) |
| Backoff multiplier | `1u32.checked_shl(exp).unwrap_or(u32::MAX)` (bit-shift, fine but obscure) | `2u32.checked_pow(exp).unwrap_or(u32::MAX)` (idiomatic 2^n) |
| Attempts counter | `job.attempts += 1` | `job.attempts.saturating_add(1)` |
| `now + delay` overflow | Not handled (would panic on `Instant + Duration::MAX`) | `now.checked_add(delay).unwrap_or_else(...)` with multi-decade fallback |
| `unwrap` in lib code | One: `1u32.checked_shl(exp).unwrap_or(...)` | One: `.expect("id came from self.jobs")` (with reason) |

Direct's `get_state` is a code smell — it pattern-matches every variant and reconstructs the same variant. The cause is forgetting to make `JobState: Copy`. Design-first added `Copy` (sensible for a five-variant unit enum) and the implementation simplifies. Design-first also handles three classes of overflow that direct doesn't notice.

Both solutions pass our tests. Both would probably ship. But if you read them as a reviewer, design-first reads like code that has thought about edge cases; direct reads like code that hasn't yet.

## Tiny scripting language

**Direct:** 550 LOC. **Design-first:** 683 LOC.

Biggest LOC delta of the three. Where does it go?

The single most visible structural difference is in the AST:

```rust
// Direct: positional enum variants
pub enum Expr {
    Int(i64),
    Bool(bool),
    Ident(String),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}
```

```rust
// Design-first: named-field enum variants
pub enum Expr {
    Int(i64),
    Bool(bool),
    Ident(String),
    Let { name: String, value: Box<Expr>, body: Box<Expr> },
    If  { cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary  { op: UnaryOp, operand: Box<Expr> },
}
```

Direct has eight variants, three of which collapse arithmetic/logical/comparison into a single `BinOp(...)` and inline two unary cases (`Neg`, `Not`) directly on `Expr`. Design-first has seven variants — but factors out a separate `UnaryOp` enum and uses **named fields** everywhere, which propagates: every match site in design-first reads like `Expr::If { cond, then_branch, else_branch }` instead of `Expr::If(c, t, e)`.

Other structural differences:

| | Direct | Design-first |
|---|---|---|
| Operator enums | `BinOp` (12 variants, mixed arithmetic/comparison/logical) | `BinOp` (12 variants, no unary) + `UnaryOp` (2 variants) |
| `Value` derives | `Clone, PartialEq` | `Clone, Copy, PartialEq, Eq` |
| Debug on errors | Manual `impl` (verbose) | `#[derive(Debug)]` |
| Doc comments | None | Every public type, every public variant |
| Section comments in source | None | `// ---------- Lexer ----------` etc. |

Where does the +133 LOC go? Roughly: ~30 lines from doc comments, ~10 from the `UnaryOp` factoring, ~30 from named-field variants spreading across match arms, ~30 from a slightly more verbose recursive-descent parser, ~30 from the lexer treating each token as its own variant rather than collapsing operators. The remaining ~10 is whitespace and section comments.

## Cross-cutting patterns

The same things show up in all three pairs:

1. **Doc comments.** Design-first systematically puts `///` comments on public items — types, variants, methods. Direct rarely does, even when the items are obvious public-API surface. Most of the LOC delta is here.

2. **Error-variant payload richness.** Design-first more often carries information in error variants (`OutOfRange(u32)`, `InvalidChar(char)`); direct leans toward unit variants and stuffs information into prose `String` fields if at all.

3. **Trait derives.** Design-first reaches for `Copy`, `Clone`, `Eq` on small types more readily. The downstream effect is real: see job_queue's `get_state` simplification.

4. **Defensive coding.** Design-first more often handles overflow with `saturating_add`, `checked_pow`, `checked_add`. Direct usually doesn't.

5. **Idiomatic helpers.** `matches!` macro, `.map(|j| j.state)`, `expect("reason")` — these appear more in design-first.

6. **Volunteered cruft.** Direct sometimes adds inline `#[cfg(test)]` tests, `impl Error` traits, helpers the spec didn't ask for. Design-first stays closer to the spec contract.

## What this changes about my read of the numbers

The LOC delta isn't "design-first is more verbose." It's roughly:
- ~50% doc comments (design-first documents; direct doesn't)
- ~25% richer types (named-field variants, factored op enums, error payloads)
- ~25% slightly more careful implementation (overflow, `Copy`, `matches!`)

The clippy delta probably **goes the other way once you weight by intent.** Direct has more clippy warnings on simple tasks even though its code is shorter, because it lacks the careful idioms that suppress pedantic lints. Design-first's higher absolute clippy count on scripting may be partly because it has more code for clippy to lint, not because it's worse.

## Bias caveats

- N=3 pairs, run_0 of each treatment. Sample is tiny.
- I knew the labels. My reading is unblinded.
- I picked things to compare *after* reading. Implicit cherry-picking.

Verdict: there's a real qualitative difference worth measuring systematically. The B2 rubric scorer (LLM-as-judge, blinded, all 18 cells per arm) is now justified — the qualitative differences this surfaced (doc coverage, error-variant richness, trait derives, defensive idioms) are concrete enough to score for.
