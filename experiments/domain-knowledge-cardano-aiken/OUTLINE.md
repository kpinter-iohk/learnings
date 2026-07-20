# domain_knowledge.md — outline + sample sections

This file is for review. It proposes a structure for the actual `domain_knowledge.md` and contains two sections written in full (the **mental model** preamble and one **anti-pattern entry**) so you can react to both the structural choices and the writing voice before the full doc is written.

---

## Design decisions to react to

1. **Ordering: mental model → validator structure → syntax → worked example → anti-patterns → stdlib (appendix).** Rationale: build the *right way* first (concepts → tools → application), then surface pitfalls, then reference material. Anti-patterns near the end so they're salient when the model is about to generate code; stdlib as appendix because it's reference, not learning.

2. **Voice: declarative, directive, no hedging.** The reader is an LLM about to write code, not a human exploring the language. "X is Y" beats "X is generally Y in most cases." Code blocks first-class; explanations *support* code, not the other way around.

3. **Length target: 600–900 lines.** RESEARCH.md is 1275 lines but heavy on verbatim quotes. The final doc distills to roughly half the size by replacing quoted material with directive statements + citations.

4. **Preamble (Section 0) tells the sub-agent *how* to use the doc.** Short — under 30 lines — but explicit: "you are writing Aiken, not Plutus or Solidity; verify stdlib names against §6; if a pattern below isn't covered, consult aiken-lang.org rather than guessing." This is borrowed from the IOG cardano-mentor agent pattern.

5. **Anti-pattern format: each entry has name → wrong code → why it's wrong → right code.** No prose paragraphs without code. The "wrong/right" diff is the most teachable structure for LLM consumption.

6. **No external dependencies in worked example.** Vesting validator uses `sidan-lab/vodka`. Using hello-world (no deps) keeps the example self-contained — sub-agents can lift patterns without needing to add dependencies.

---

## Proposed section structure

```
0. How to use this document                          ~25 lines
1. The eUTXO mental model                            ~80 lines
   1.1 What Cardano is and isn't (Ethereum contrast)
   1.2 Inputs, outputs, datums, redeemers
   1.3 Validators are predicates
   1.4 The six script purposes (only spend gets a datum)
   1.5 Reference inputs vs spending inputs
2. Validator structure                               ~100 lines
   2.1 The validator block
   2.2 Handler signatures (table)
   2.3 Datum is always Option<T>
   2.4 The else(_ctx: ScriptContext) fallback
   2.5 Validator parameters
   2.6 Calling handlers from tests
3. Aiken syntax cheatsheet                           ~120 lines
   3.1 Primitive types
   3.2 Custom types (records, ADTs, generics, aliases)
   3.3 Pattern matching (when..is, expect, if..is)
   3.4 Functions (pub vs fn, labelled args, capture)
   3.5 Pipes
   3.6 Modules and imports
   3.7 Traces, ? operator, fail, todo
4. Worked example: hello_world validator             ~80 lines
   4.1 Full annotated code
   4.2 What each pattern teaches
   4.3 How tests work
5. Anti-pattern catalog                              ~250 lines
   5.1 Double satisfaction
   5.2 Opaque-Data misuse
   5.3 Datum confusion across purposes
   5.4 Trace level affects script hash
   5.5 Datum continuity in state machines
   5.6 STT / beacon spoofing
   5.7 On-chain / off-chain encoding drift
   5.8 Floating-point (Aiken has none)
   5.9 Narrow validity intervals
   5.10 Reserialization breaking hashes
   5.11 Policy-id forgetting language tag
   5.12 Forwarding-validation (positive recommendation)
   5.13 Byron addresses in Plutus context
6. Stdlib anchor list (appendix)                     ~80 lines
   6.1 Module index
   6.2 Most-used modules with function names
A. Project structure & toolchain (appendix)          ~40 lines
```

**Total estimate: ~775 lines.** Within the 600–900 target.

---

## SAMPLE — Section 0: How to use this document

```markdown
# Cardano smart contracts in Aiken — domain knowledge

You are about to write Aiken code for Cardano. This document gives you the
mental model, syntax, and pitfall catalog you need. Read it before generating
any validator code.

## What this document is

- A bootstrap for Cardano's eUTXO model and the Aiken language as it exists today
- A whitelist of stdlib function names (§6) — do not guess function names; cite
  this list or read aiken-lang.org
- A catalog of mistakes LLMs make in this domain (§5) — actively check your
  output against each

## What this document is not

- Not Solidity. Not Plutus-Tx. Not Marlowe. If you import patterns from any of
  those, you will write wrong code.
- Not a tutorial. It assumes you can already write functional code in a typed
  language.
- Not exhaustive. When in doubt, fetch from aiken-lang.org rather than guess.

## When to verify rather than recall

- Stdlib function signatures beyond the names in §6
- The exact field order of types in `cardano/transaction.{Transaction, Output, Input}`
- Any feature added after Plutus V3 (governance, voting, treasury)

If you're about to use a function not listed in §6, stop and verify it exists.
```

---

## SAMPLE — Section 1 in full: The eUTXO mental model

```markdown
## 1. The eUTXO mental model

### 1.1 Cardano is not Ethereum

Cardano's state is the set of **unspent transaction outputs** (UTxOs). There is
no per-address mutable balance, no global storage, no contract state. A
transaction consumes some UTxOs and creates new ones. State changes are encoded
by *replacing* a UTxO with a new one whose datum carries the new state.

If you find yourself writing code that mutates a value, reads a "current state"
without consuming a UTxO, or assumes reentrancy is possible — stop. Those
concepts do not exist here.

The standard metaphor is a wall of post-it notes:

> An input is a reference to a previous output. Think of outputs as post-it
> notes with a unique serial number and inputs as being this serial number. A
> transaction is a document indicating which post-it notes should be destroyed
> and which new ones should be pinned to the wall.

(— aiken-lang.org/fundamentals/eutxo)

### 1.2 Inputs, outputs, datums, redeemers

Every UTxO carries:

- An **address** — who or what can spend it
- A **value** — ADA + native tokens
- A **datum** — arbitrary on-chain data attached when the UTxO is created

When a UTxO is spent, the transaction supplies a **redeemer** — arbitrary data
provided at spending time. The validator at the UTxO's script address receives:

- The datum (set at creation time)
- The redeemer (provided at spending time)
- The script context (the transaction plus a `ScriptInfo` describing the
  current invocation's purpose and target)

Parametric-function analogy:

```
              Script
           ╭─────────╮
     f(x) = x * a + b  = true | false
            ╿   ╿   ╿
   Redeemer ┘   │   │
                └─┬─┘
                Datum
```

The script defines the function shape; the datum parameterises it; the
redeemer (plus the transaction) is the argument.

### 1.3 Validators are predicates

A validator returns `True` or `False`. `True` authorizes the action; `False`
(or a `fail`, or a failed `expect`) rejects the transaction.

Validators **do not modify state**. They inspect the transaction and approve
or reject. To "update" state, a validator approves a transaction that consumes
the old state-carrying UTxO and creates a new one with the new datum. The
validator's job is to check that the new UTxO is well-formed.

### 1.4 The six script purposes

A validator handler is named for its purpose. Only one of them receives a
datum:

| Purpose    | Controls                                  | Receives datum? |
|------------|-------------------------------------------|-----------------|
| `spend`    | Spending a UTxO                           | **Yes** (`Option<T>`) |
| `mint`     | Minting/burning native tokens             | No |
| `withdraw` | Withdrawing staking rewards               | No |
| `publish`  | Publishing delegation certificates        | No |
| `vote`     | Voting on governance proposals            | No |
| `propose`  | Constitutional guardrails (single-script) | No |

If you write a `mint` handler that tries to receive a datum, the code will not
compile. If a minting policy needs data, it must locate the relevant input or
reference input in `self.inputs` / `self.reference_inputs` and read that
input's `output.datum`.

The datum in `spend` is always `Option<T>`, never `T`. Anyone can lock funds
at a script address without supplying a datum, so the optional wrapper is
forced. The standard idiom is to halt if the datum is missing:

```aiken
spend(datum_opt: Option<MyDatum>, redeemer: MyRedeemer, _ref: OutputReference, self: Transaction) {
  expect Some(datum) = datum_opt
  // ... rest of validator
}
```

### 1.5 Reference inputs vs spending inputs

A transaction has two input lists:

- `inputs: List<Input>` — UTxOs being **consumed** by this transaction. Each
  one runs its address's validator.
- `reference_inputs: List<Input>` — UTxOs being **read** but not consumed.
  Their datums and values are visible to validators but they remain on the
  wall.

Reference inputs (CIP-31) are how oracles, parameter dictionaries, and shared
script references are distributed without recreating a UTxO on every use. A
validator reading reference inputs must verify the inputs carry an authentic
identifier (typically an NFT — see §5.6 STT pattern), since anyone can create
a UTxO with arbitrary datum.
```

---

## SAMPLE — One anti-pattern entry in full: §5.1 Double satisfaction

```markdown
### 5.1 Double satisfaction

**The mistake.** A spending validator checks "≥ N ADA was paid to Bob" and
runs once per spent input. If two UTxOs at the same script address are spent
in one transaction, both validators see the same "Bob received N ADA" output,
and a single payment satisfies both.

**Vulnerable validator:**

```aiken
validator exploitable_swap {
  spend(optional_datum: Option<DatumSwap>, _redeemer: Data, _own_ref: OutputReference, self: Transaction) {
    expect Some(datum) = optional_datum
    let beneficiary = address.from_verification_key(datum.beneficiary)
    let user_outputs =
      list.filter(self.outputs, fn(o) { o.address == beneficiary })
    let value_paid =
      list.foldl(user_outputs, assets.zero, fn(o, total) { merge(o.value, total) })
    (lovelace_of(value_paid) >= datum.price)?
  }
}
```

Spend two UTxOs at this address in one transaction, each expecting 100 ADA to
Bob. Pay Bob 100 ADA once. Both validators independently see that 100 ADA was
paid to Bob. Both pass. You drained 200 ADA for 100.

**The fix.** Tag the output. Embed `own_ref` (the input reference) in the
output's datum and require a 1-to-1 match between the spent input and a
dedicated payment output:

```aiken
validator swap {
  spend(optional_datum: Option<DatumSwap>, _redeemer: Data, own_ref: OutputReference, self: Transaction) {
    expect Some(datum) = optional_datum
    let beneficiary = address.from_verification_key(datum.beneficiary)
    let user_outputs_restricted =
      list.filter(self.outputs, fn(output) {
        when output.datum is {
          InlineDatum(output_datum) ->
            if output_datum is OutputReference {
              and {
                output.address == beneficiary,
                own_ref == output_datum,
              }
            } else { False }
          _ -> False
        }
      })
    let value_paid =
      list.foldl(user_outputs_restricted, assets.zero, fn(n, acc) { merge(n.value, acc) })
    (lovelace_of(value_paid) >= datum.price)?
  }
}
```

Now each input demands its *own* payment output, identified by its
`OutputReference`. Double-spending no longer collapses to a single payment.

**Why the foreign-output check uses `if..is`, not `expect`.** Other unrelated
outputs in the transaction may have differently-shaped datums. `expect` would
reject the whole transaction; `if..is` lets us skip outputs we don't care
about.

(Sources: aiken-lang.org/fundamentals/common-design-patterns;
vacuumlabs writeup on double satisfaction.)
```

---

## What I want you to react to

- **Structure:** does the section order make sense? Anything missing?
- **Length:** OK with ~775 lines, or should I push for tighter?
- **Voice:** is the directive tone right? Too terse? Right amount of citation?
- **Sample section 1:** does the mental-model section hit the right level of
  abstraction for an LLM bootstrap? Anything you'd add or cut?
- **Sample anti-pattern:** is the wrong-code → why → right-code → footnote
  format right for the catalog? Should I include the "source" footnote in each
  entry or just trust the reader?
