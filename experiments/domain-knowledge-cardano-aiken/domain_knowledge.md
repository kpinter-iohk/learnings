# Cardano smart contracts in Aiken — domain knowledge

You are about to write Aiken code for Cardano. This document gives you the mental model, syntax, and pitfall catalog you need. Read it before generating any validator code.

## 0. How to use this document

**What this is.** A bootstrap for Cardano's eUTXO model and the Aiken language as it exists today. A whitelist of stdlib function names (§6) — do not guess function names; cite this list or fetch from aiken-lang.org. A catalog of mistakes LLMs make in this domain (§5) — actively check your output against each.

**What this is not.** Not Solidity. Not Plutus-Tx. Not Marlowe. If you import patterns from any of those, you will write wrong code. Not a tutorial — assumes you can already write functional code in a typed language. Not exhaustive — when in doubt, fetch from aiken-lang.org rather than guess.

**Verify before recalling.** Stdlib function signatures beyond the names in §6, the exact field order in `cardano/transaction` types, and any feature added after Plutus V3 (governance, voting, treasury) — verify these against aiken-lang.org or `aiken-lang/stdlib` on GitHub before using them. If you are about to use a function not listed in §6, stop and verify it exists.

**Vocabulary discipline.** Use "eUTxO" not "UTXO" when referring to Cardano's model. Use "spending validator" not "smart contract" when being specific. Use "minting policy" not "mint contract" or "token contract." Use "datum" for data attached to a UTxO and "redeemer" for data provided at spend time — they are not interchangeable.

---

## 1. The eUTxO mental model

### 1.1 Cardano is not Ethereum

Cardano's state is the set of **unspent transaction outputs** (UTxOs). There is no per-address mutable balance, no global storage, no contract state. A transaction consumes some UTxOs and creates new ones. State changes are encoded by *replacing* a UTxO with a new one whose datum carries the new state.

If you find yourself writing code that mutates a value, reads a "current state" without consuming a UTxO, or assumes reentrancy is possible — stop. Those concepts do not exist here.

The standard metaphor is a wall of post-it notes:

> An input is a reference to a previous output. Think of outputs as post-it notes with a unique serial number and inputs as being this serial number. A transaction is a document indicating which post-it notes should be destroyed and which new ones should be pinned to the wall.

The blockchain state is "the entire wall of remaining post-it notes." Each input is consumed exactly once — the protocol guarantees this, so concurrent spends of the same UTxO are impossible by construction.

### 1.2 Inputs, outputs, datums, redeemers

Every UTxO carries:

- An **address** — who or what can spend it (a public-key hash or a script hash)
- A **value** — ADA + native tokens
- A **datum** — arbitrary on-chain data attached when the UTxO is created

When a UTxO is spent, the transaction supplies a **redeemer** — arbitrary data provided at spending time. The validator at the UTxO's script address receives the datum (set at creation), the redeemer (set at spending), and the transaction itself.

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

The script defines the function shape; the datum parameterises it; the redeemer (plus the transaction) is the argument.

### 1.3 Validators are predicates

A validator returns `True` or `False`. `True` authorizes the action; `False`, a `fail`, or a failed `expect` rejects the transaction.

Validators **do not modify state**. They inspect the transaction and approve or reject. To "update" state, a validator approves a transaction that consumes the old state-carrying UTxO and creates a new one with the new datum. The validator's job is to check that the new UTxO is well-formed.

Public-key authorisation is a special case of the same model: the public-key hash is the datum, the signature is the redeemer, and the verification algorithm is the script.

### 1.4 The six script purposes

A validator handler is named for its purpose. Only one of them receives a datum:

| Purpose    | Controls                                    | Receives datum? |
|------------|---------------------------------------------|-----------------|
| `spend`    | Spending a UTxO                             | **Yes** (`Option<T>`) |
| `mint`     | Minting/burning native tokens               | No |
| `withdraw` | Withdrawing staking rewards                 | No |
| `publish`  | Publishing delegation certificates          | No |
| `vote`     | Voting on governance proposals              | No |
| `propose`  | Constitutional guardrails (single ledger-wide script) | No |

If you write a `mint` handler that tries to receive a datum, the code will not compile. If a minting policy needs data, it must locate the relevant input or reference input in `self.inputs` / `self.reference_inputs` and read that input's `output.datum`.

The datum in `spend` is **always** `Option<T>`, never `T`. Anyone can lock funds at a script address without supplying a datum, so the optional wrapper is forced. The standard idiom is to halt if the datum is missing:

```aiken
spend(datum_opt: Option<MyDatum>, redeemer: MyRedeemer, _ref: OutputReference, self: Transaction) {
  expect Some(datum) = datum_opt
  // ... rest of validator
}
```

### 1.5 Reference inputs vs spending inputs

A transaction has two input lists:

- `inputs: List<Input>` — UTxOs being **consumed** by this transaction. Each one runs its address's validator.
- `reference_inputs: List<Input>` — UTxOs being **read** but not consumed. Their datums and values are visible to validators but they remain on the wall.

Reference inputs (CIP-31) are how oracles, parameter dictionaries, and shared script references are distributed without recreating a UTxO on every use. A validator reading reference inputs must verify the inputs carry an authentic identifier (typically an NFT — see §5.6 STT pattern), since anyone can create a UTxO with arbitrary datum.

### 1.6 The ScriptContext

In Aiken handlers, the transaction is passed as `self: Transaction` and a per-purpose target is passed alongside (a `PolicyId` for mint, an `OutputReference` for spend, a `Credential` for withdraw, etc. — see §2.2).

A handler can also be written as a fallback that receives the full untyped `ScriptContext` — see §2.4.

---

## 2. Validator structure

### 2.1 The validator block

A validator is a named block containing one or more handlers. The handler name must be one of the six purposes:

```aiken
validator my_script {
  spend(datum: Option<MyDatum>, redeemer: MyRedeemer, utxo: OutputReference, self: Transaction) {
    // body returns Bool
  }
}
```

All handlers in one `validator` block share the same compiled script hash. That property is what enables the withdraw-zero forwarding pattern (§5.12).

### 2.2 Handler signatures

Every handler takes a redeemer, a target, and the transaction. The `spend` handler takes a fourth argument — the datum — *before* the others. The target type varies by purpose:

| Purpose    | Signature |
|------------|-----------|
| `spend`    | `spend(datum: Option<D>, redeemer: R, utxo: OutputReference, self: Transaction)` |
| `mint`     | `mint(redeemer: R, policy_id: PolicyId, self: Transaction)` |
| `withdraw` | `withdraw(redeemer: R, account: Credential, self: Transaction)` |
| `publish`  | `publish(redeemer: R, certificate: Certificate, self: Transaction)` |
| `vote`     | `vote(redeemer: R, voter: Voter, self: Transaction)` |
| `propose`  | `propose(redeemer: R, proposal: ProposalProcedure, self: Transaction)` |

A multi-purpose validator declares several handlers in the same block:

```aiken
use cardano/address.{Credential}
use cardano/assets.{PolicyId}
use cardano/certificate.{Certificate}
use cardano/transaction.{Transaction, OutputReference}

validator my_script {
  mint(redeemer: MyMintRedeemer, policy_id: PolicyId, self: Transaction) { todo }
  spend(datum: Option<MyDatum>, redeemer: MySpendRedeemer, utxo: OutputReference, self: Transaction) { todo }
  withdraw(redeemer: MyWithdrawRedeemer, account: Credential, self: Transaction) { todo }
}
```

### 2.3 Datum is always Option<T>

In `spend`, the datum parameter is `Option<MyDatum>`, never `MyDatum`. Because the ledger cannot prevent someone from locking funds at the script address without a datum, the compiler forces you to handle the missing case.

The canonical pattern is to fail loudly:

```aiken
expect Some(datum) = datum_opt
```

The "missing-datum" branch is unreachable when the validator's intended usage is followed, so failing is correct. Be deliberate if you actually want to accept a missing datum.

### 2.4 The else fallback

A validator may add a single `else(_ctx: ScriptContext)` clause as a catch-all for any purpose not explicitly declared. It receives the full untyped `ScriptContext` and you must recover the redeemer/datum manually.

```aiken
use cardano/script_context.{ScriptContext}

validator my_script {
  spend(datum: Option<MyDatum>, redeemer: MyRedeemer, utxo: OutputReference, self: Transaction) {
    expect Some(d) = datum
    todo
  }
  else(_ctx: ScriptContext) {
    fail @"unsupported purpose"
  }
}
```

If no `else` is provided, an undeclared purpose returns `False` (rejects).

### 2.5 Validator parameters

A validator can be parameterised. Parameters are baked into the compiled script — different parameter values produce different script hashes, and therefore different addresses or policy IDs.

```aiken
validator one_shot_policy(utxo_ref: OutputReference) {
  mint(_redeemer: Data, _policy_id: PolicyId, self: Transaction) {
    expect list.any(self.inputs, fn(input) { input.output_reference == utxo_ref })
    True
  }
}
```

Parameters are how you make a policy "one-shot": parameterise it by an `OutputReference` and require that UTxO to be consumed when minting. Once consumed, no one can mint under that policy again.

Apply parameters at build time with `aiken blueprint apply`.

### 2.6 Calling handlers from tests

Handlers are callable like regular functions. The validator parameters come first, then the handler arguments:

```aiken
test happy_path() {
  let utxo_ref = mock_utxo_ref(0, 1)
  let redeemer = MyRedeemer { ... }
  let policy_id = mock_policy_id(1)
  let transaction = mock_tx()
  one_shot_policy.mint(utxo_ref, redeemer, policy_id, transaction)
  //              ^^                  validator parameter first, then handler args
}
```

Run tests with `aiken check`. Tests live in any `.ak` file under `lib/` or `validators/`; the keyword is `test`.

---

## 3. Aiken syntax cheatsheet

### 3.1 Primitive types

| Type        | Literal syntax |
|-------------|----------------|
| `Bool`      | `True`, `False` |
| `Int`       | `42`, `1_000_000`, `0xF`, `0b1111` (arbitrary precision) |
| `ByteArray` | `#[10, 255]` (byte list), `"foo"` (utf-8 → bytes), `#"666f6f"` (hex) |
| `String`    | `@"hello"` (note the `@` — strings exist only for tracing) |
| `List<a>`   | `[1, 2, 3]`, `[1, ..[2, 3]]` |
| `Tuple`     | `(10, "hi")`, `(1, 4, [0])` |
| `Pair`      | `Pair(14, "aiken")` |
| `Option<a>` | `Some(a)`, `None` |
| `Ordering`  | `Less`, `Equal`, `Greater` |
| `Void`      | `Void` |
| `Data`      | opaque, any serialisable value |
| `Never`     | identical to `None` |

**There is no floating-point type.** `Int` is arbitrary-precision. For fractions, use `aiken/math/rational` or integer scaling (e.g. basis points × 10000).

**Strings vs ByteArrays.** Use `ByteArray` for actual data (datums, hashes, asset names). Use `String` (with the `@` prefix) only inside `trace`, `fail`, `todo`, or `expect` messages.

### 3.2 Custom types

Records with named fields (single-constructor shorthand):

```aiken
type Datum {
  signer: ByteArray,
  count: Int,
}
```

Sum types (ADTs) with multiple constructors:

```aiken
type User {
  LoggedIn { username: ByteArray }
  Guest
}
```

Generics:

```aiken
type Box<inner_type> {
  Box(inner: inner_type)
}
```

Type aliases:

```aiken
type MyNumber = Int
type Person = (String, Int)
```

Record-update syntax:

```aiken
Person { ..person, age: person.age + 1 }
```

`pub opaque type` exports the type name but hides constructors and fields — clients can hold values but cannot inspect or build them directly.

PlutusData encoding can be controlled with annotations:

```aiken
@list
type Datum {
  signer: ByteArray,
  count: Int,
}

type Bool {
  @tag(1) True
  @tag(0) False
}
```

`@list` forces `PlutusList` encoding instead of `Constr`. `@tag(n)` pins constructor indices. Use these only when the off-chain side requires a specific encoding (§5.7).

### 3.3 Pattern matching

`when..is` — exhaustive matching:

```aiken
fn describe(user: User) -> String {
  when user is {
    LoggedIn { username: _ } -> @"logged in"
    Guest -> @"guest"
  }
}
```

The compiler enforces exhaustiveness. Use `_` for wildcards, `|` for alternation, `..` to ignore remaining fields, `[head, ..tail]` for list destructuring.

`expect` — non-exhaustive match, halts on mismatch:

```aiken
expect Some(y) = x
expect my_datum: MyDatum = data
```

Use `expect` when failure is intended to reject the transaction.

`if..is` — soft cast, returns `Bool`:

```aiken
if d is Foo {
  d.foo == 1
} else {
  False
}
```

Use `if..is` when you want to *check* a shape without aborting (e.g., filtering foreign outputs in §5.1).

### 3.4 Functions

```aiken
fn add(x: Int, y: Int) -> Int { x + y }   // module-private

pub fn identity(x: a) -> a { x }          // exported

let inc = fn(x) { x + 1 }                 // anonymous

let add_one = add(1, _)                   // capture / partial application

// labelled arguments
fn replace(self: String, pattern: String, replacement: String) { ... }
replace(self: @"A,B,C", pattern: @",", replacement: @" ")
```

`pub fn` exports the function; bare `fn` is module-private. Recursive anonymous functions are not supported — use top-level definitions.

### 3.5 Pipes

```aiken
string
  |> string_builder.from_string
  |> string_builder.reverse
  |> string_builder.to_string
```

Each line applies the function to the result of the previous line. Combines with capture:

```aiken
1 |> add(3) |> add(6) |> add(9)
```

### 3.6 Modules and imports

A file at `lib/straw_hats/sunny.ak` is the module `straw_hats/sunny`. `pub` exports.

```aiken
use straw_hats/sunny                          // qualified
use animal/dog as kitty                       // alias
use animal/dog.{Dog, stroke}                  // unqualified imports
use animal/dog.{Dog, stroke} as kitty         // both
```

The `aiken` prelude is automatically available. `aiken/builtin` exposes raw Plutus core builtins (use these only as a last resort). The `env/` directory holds environment-specific modules — reference them with `use env` regardless of filename.

### 3.7 Traces, the `?` operator, fail, todo

```aiken
trace @"redeemer received": string.from_bytearray(redeemer.msg)
```

Traces appear in `aiken check` output but do not affect on-chain behaviour. **Traces are stripped by `aiken build` by default.** Including or excluding traces changes the compiled script bytes and therefore the script hash — see §5.4.

The `?` operator traces a sub-expression only when it evaluates to `False`:

```aiken
must_say_hello? && must_be_signed?
```

This is the idiomatic way to write a validator's final return — every failed condition leaves a trace.

`fail` and `todo`:

```aiken
fail @"reason"   // halts execution; no compiler warning
todo @"note"     // halts; compiler emits "not yet implemented" warning
```

`fail` is for intentional rejection. `todo` is a placeholder that signals the code is incomplete — the compiler warning prevents you from shipping it accidentally.

A doc-comment on an `expect` line produces a runtime trace when the expectation fails:

```aiken
/// life, universe and everything.
expect answer == 42
```

---

## 4. Worked example: hello_world validator

This is the canonical minimal Aiken validator. It demonstrates the standard patterns: datum destructuring, signature checking, redeemer comparison, the `?` trace operator, and the `else` fallback.

```aiken
use aiken/collection/list
use aiken/crypto.{VerificationKeyHash}
use aiken/primitive/string
use cardano/script_context.{ScriptContext}
use cardano/transaction.{OutputReference, Transaction}

pub type Datum {
  owner: VerificationKeyHash,
}

pub type Redeemer {
  msg: ByteArray,
}

validator hello_world {
  spend(
    datum: Option<Datum>,
    redeemer: Redeemer,
    _own_ref: OutputReference,
    self: Transaction,
  ) {
    trace @"redeemer": string.from_bytearray(redeemer.msg)
    expect Some(Datum { owner }) = datum
    let must_say_hello = redeemer.msg == "Hello, World!"
    let must_be_signed = list.has(self.extra_signatories, owner)
    must_say_hello? && must_be_signed?
  }

  else(_ctx: ScriptContext) {
    False
  }
}

test hello_world_example() {
  let datum =
    Datum { owner: #"00000000000000000000000000000000000000000000000000000000" }
  let redeemer = Redeemer { msg: "Hello, World!" }
  let placeholder_utxo = OutputReference { transaction_id: "", output_index: 0 }
  hello_world.spend(
    Some(datum),
    redeemer,
    placeholder_utxo,
    Transaction { ..transaction.placeholder, extra_signatories: [datum.owner] },
  )
}
```

### What each line teaches

- `use aiken/collection/list` — stdlib modules are imported with their full path. The module here exposes `list.has` (see §6).
- `use cardano/transaction.{OutputReference, Transaction}` — unqualified imports of specific items.
- `pub type Datum { ... }` — `pub` so the type appears in the generated `plutus.json` blueprint. Off-chain builders need this.
- `_own_ref: OutputReference` — underscore prefix because the variable is unused. Aiken warns on unused variables.
- `expect Some(Datum { owner }) = datum` — fail fast if the datum is missing or malformed. Destructures the inner record in one step.
- `must_say_hello? && must_be_signed?` — the `?` operator. If the validator fails, the trace says exactly which condition went wrong.
- `else(_ctx: ScriptContext) { False }` — explicit fallback that rejects any non-spend invocation. Without this, the validator would still reject other purposes (default behaviour) but the explicit form documents intent.
- `transaction.placeholder` — a stdlib-provided default `Transaction` for tests. Spread (`..`) lets you override only the fields the test cares about.

### What this example does not show

- Datum continuity (§5.5): there is no "continuing output" check because nothing in this contract carries state across transactions.
- Reference inputs: not used.
- Minting: not used.
- Validity range / time bounds: not used. For time-based validators see §5.9.

---

## 5. Anti-pattern catalog

These are mistakes documented in the Aiken docs, the IOG cardano-mentor agent, and published security writeups. Check each one against your output before returning code.

### 5.1 Double satisfaction

**The mistake.** A spending validator checks "≥ N ADA was paid to Bob" and runs once per spent input. If two UTxOs at the same script address are spent in one transaction, both validators see the same "Bob received N ADA" output, and a single payment satisfies both.

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

Spend two UTxOs at this address in one transaction, each expecting 100 ADA to Bob. Pay Bob 100 ADA once. Both validators independently see that 100 ADA was paid to Bob. Both pass. You drained 200 ADA for 100.

**The fix.** Tag the output. Embed `own_ref` (the input reference) in the output's datum and require a 1-to-1 match between the spent input and a dedicated payment output:

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

Each input demands its own payment output, identified by its `OutputReference`. Double-spending no longer collapses to a single payment.

The foreign-output check uses `if..is`, not `expect`, because other outputs in the transaction may have differently-shaped datums. `expect` would reject the whole transaction; `if..is` skips outputs the validator doesn't care about.

### 5.2 Opaque-Data misuse

**The mistake.** Treating an arbitrary `Data` value as a specific type without going through `expect` or `if..is`.

Upcasting (custom type → `Data`) is implicit and safe. **Downcasting (`Data` → custom type) can fail at runtime** because `Data` can hold anything serialisable. Forgetting this leads to silent acceptance of malformed datums or unhelpful run-time failures with no explanation.

**The fix.** Always be explicit about the cast direction:

```aiken
// Halt on bad shape — appropriate when you require the datum to be present
expect my_datum: MyDatum = data

// Soft cast — appropriate when foreign data may not match and you want to skip it
if data is MyDatum {
  ...
}
```

Use `expect` when you control the data source. Use `if..is` when filtering data you don't control (e.g., scanning all transaction outputs for a specific shape, as in §5.1).

### 5.3 Datum confusion across purposes

**The mistake.** Writing a `mint`, `withdraw`, `publish`, `vote`, or `propose` handler that tries to receive or use a datum.

Only `spend` handlers receive a datum (and only as `Option<T>`). The other five purposes operate on credentials, policies, certificates, voters, or proposals — they have no associated UTxO datum.

**The fix.** If a minting policy needs to read data, it must locate the relevant input in `self.inputs` or `self.reference_inputs` and inspect that input's `output.datum`:

```aiken
mint(redeemer: R, policy_id: PolicyId, self: Transaction) {
  expect Some(stt_input) =
    list.find(self.inputs, fn(i) { list.has(assets.policies(i.output.value), policy_id) })
  expect InlineDatum(d) = stt_input.output.datum
  expect state: MyState = d
  // ... use `state` ...
}
```

### 5.4 Trace level affects script hash

**The mistake.** Comparing addresses computed under different trace levels, or leaving traces enabled in `aiken build` when you intended the production hash.

Including or excluding traces does not change the semantic of your program, but it changes the compiled UPLC bytes and therefore the script hash and the derived address.

**The fix.**
- `aiken build` strips traces by default — this is the production hash.
- `aiken check` preserves traces by default — this is the testing hash.
- Use `--trace-level silent` to strip traces in `aiken check` if you need hash parity.
- Use `--trace-level compact` in `aiken build` if you want minimal-cost production traces. Compact preserves only the label before `:` in each `trace` call.
- Never rely on an address computed during `aiken check` for deployment.

### 5.5 Datum continuity in state machines

**The mistake.** When a UTxO holding state is consumed and a new one is created to replace it, the new datum drops an immutable field, mutates one that should be invariant, or fails to constrain a mutable one (e.g., a counter must increment by exactly 1).

State machines on eUTxO are encoded as input-datum → output-datum transitions. A break in continuity lets an attacker forge a new state.

**The fix.** Explicit per-field checks on every continuing-UTxO transition:

```aiken
spend(datum: Option<State>, _r: Data, own_ref: OutputReference, self: Transaction) {
  expect Some(in_state) = datum
  expect Some(out) =
    list.find(self.outputs, fn(o) { o.address == own_input_address(self, own_ref) })
  expect InlineDatum(out_data) = out.datum
  expect out_state: State = out_data
  and {
    out_state.immutable_field == in_state.immutable_field,
    out_state.counter == in_state.counter + 1,
    out_state.next_allowed_slot >= in_state.next_allowed_slot,
  }
}
```

For each field on the continuing datum, ask: is it immutable (must equal), monotonic (must `>=`), or bounded (must satisfy some transition rule)? There is no default; every field needs a constraint.

### 5.6 STT / beacon spoofing

**The mistake.** Relying on an input's datum without proving the input carries a uniqueness-anchoring NFT.

Anyone can create a UTxO with arbitrary datum at any address. Without a State Thread Token (an NFT minted under a one-shot policy), the validator may accept a forged input claiming to be the "real" state UTxO.

**The fix.** Mint an NFT under a one-shot policy parameterised by an `OutputReference`. Require all state-bearing inputs and outputs to carry that NFT. Verify both the mint quantity (must equal 1) and that the parameter UTxO was actually consumed at mint time:

```aiken
validator counter_stt(utxo_ref: OutputReference) {
  mint(_redeemer: Data, policy_id: PolicyId, self: Transaction) {
    let Transaction { inputs, outputs, mint, .. } = self
    expect [Pair(_asset_name, quantity)] =
      mint |> assets.tokens(policy_id) |> dict.to_pairs()
    let is_output_consumed =
      list.any(inputs, fn(input) { input.output_reference == utxo_ref })
    expect Some(nft_output) =
      list.find(outputs, fn(o) { list.has(assets.policies(o.value), policy_id) })
    expect InlineDatum(datum) = nft_output.datum
    expect counter: Int = datum
    is_output_consumed? && (1 == quantity)? && counter == 0
  }

  spend(_datum: Option<Data>, _r: Data, own_ref: OutputReference, self: Transaction) {
    let Transaction { inputs, outputs, .. } = self
    expect Some(own_input) = list.find(inputs, fn(i) { i.output_reference == own_ref })
    expect Script(own_script_hash) = own_input.output.address.payment_credential
    expect Some(stt_input) =
      list.find(inputs, fn(i) { list.has(assets.policies(i.output.value), own_script_hash) })
    expect InlineDatum(input_datum) = stt_input.output.datum
    expect counter_input: Int = input_datum
    expect Some(stt_output) =
      list.find(outputs, fn(o) { list.has(assets.policies(o.value), own_script_hash) })
    expect InlineDatum(output_datum) = stt_output.datum
    expect counter_output: Int = output_datum
    expect stt_input.output.address == stt_output.address
    counter_output == counter_input + 1
  }
}
```

Because `utxo_ref` is a parameter, the policy ID is unique to the deployment. Once `utxo_ref` is consumed, the policy can never mint again — the NFT is genuinely one-of-a-kind.

### 5.7 On-chain / off-chain encoding drift

**The mistake.** Off-chain transaction builder (Rust, TypeScript, Python) encodes a datum or redeemer whose field order, optional wrapping, or type widths disagree with the Aiken type.

The validator decodes the on-chain `Data` against its declared type. A field-order swap or missing `Option` wrapper causes `expect` to fail at runtime, or worse — silently parses into the wrong constructor.

**The fix.**
- Treat the Aiken type as the canonical schema.
- Use the `plutus.json` blueprint (CIP-0057) generated by `aiken build` to drive off-chain code generation.
- Use `@tag(n)` on each constructor to pin indices when the off-chain side requires stable encoding.
- Use `@list` on the type when the off-chain side expects a `PlutusList` instead of a `Constr`.
- Cross-test: round-trip a sample datum through off-chain encoding → on-chain decoding and assert equality before deploying.

### 5.8 Floating-point arithmetic does not exist

**The mistake.** Assuming Aiken has `Float`, decimals, or implicit fractional arithmetic.

`Int` is the only numeric type, and it is arbitrary-precision integer. Writing `price * 0.15` does not compile and there is no implicit conversion.

**The fix.** Either scale (e.g., basis points × 10000) or use `aiken/math/rational`:

```aiken
use aiken/math/rational

// 15% as a rational
let rate = rational.from_int(15) |> rational.div(rational.from_int(100))
```

Watch for overflow in fraction arithmetic — `aiken/math/rational` uses integer numerator/denominator, and large products can grow quickly. Bound inputs explicitly.

### 5.9 Narrow validity intervals

**The mistake.** Setting a transaction's validity interval to a 1- or 2-slot window.

Blocks are produced roughly every 20 seconds on average. A very tight validity window may not land in any block. Validators see the *bounds* of the interval, not the exact slot, so tightening the window to "now" gives the validator no extra information.

**The fix.** Use wider intervals — tens of seconds to minutes — unless you have a specific reason. Remember a lower bound `A` only proves "current time ≥ A" to the validator. An upper bound `B` only proves "current time ≤ B". Validators cannot read a current slot directly.

### 5.10 Reserialisation drift

**The mistake.** Off-chain code deserialises a transaction or datum, modifies or inspects it, then re-serialises before computing a hash.

There is no canonical serialisation of objects on Cardano. Two valid serialisations of the same object can produce different bytes — and therefore different hashes.

**The fix.** Carry the original observed bytes through your pipeline. Hash only the bytes you actually saw on-chain. Never reserialise a deserialised value if you intend to compare hashes.

### 5.11 Policy-id miscalculation

**The mistake.** Computing a policy id as `blake2b_224(serialized_script)` without the language discriminator byte.

Raw scripts are not the exact preimage of their hash. Before hashing, scripts are prefixed with a one-byte language tag:

- Native → `0x00`
- Plutus V1 → `0x01`
- Plutus V2 → `0x02`
- Plutus V3 → `0x03`

**The fix.** Always prefix with the correct version byte before `blake2b_224`. In practice, prefer reading the hash from the `plutus.json` blueprint that `aiken build` emits — it is already correct. Manual hashing is only needed in off-chain code that constructs policies on the fly.

### 5.12 Use withdraw-zero for shared logic

**Positive recommendation.** When many UTxOs at the same address need to run identical authorisation logic, running it in every `spend` handler is expensive — execution cost scales linearly with the number of inputs.

**The pattern.** Put the heavy logic in a `withdraw` handler. Each `spend` handler does only one thing: assert that the transaction also invokes the same script's `withdraw` purpose (with 0 lovelace withdrawal). The `withdraw` handler runs **once per transaction** regardless of input count.

It is always legal to withdraw 0 lovelace, so this pattern adds no fee burden beyond a single extra script execution.

This pattern is also good for correctness: the shared logic exists in exactly one place, so copy-paste bugs across `spend` handlers become impossible.

### 5.13 Byron addresses are forbidden

**The mistake.** Off-chain code constructs a Plutus-script transaction that includes Byron-era addresses as inputs or outputs.

The ledger rejects any transaction containing a Plutus script invocation when any output or input UTxO is locked by a Byron address.

**The fix.** Filter Byron addresses out before constructing the transaction. Treat them as deprecated.

### 5.14 Smaller traps

- **Aiken is not Rust.** The compiler is written in Rust; the language is not. Do not reach for Rust idioms (`?` for error propagation, `Result<T, E>`, `match`, lifetimes — none of these exist).
- **Pattern matches must be exhaustive.** The compiler enforces this.
- **`fail` versus `todo`.** Both halt at runtime. `todo` emits a compile-time warning so you cannot ship it accidentally.
- **Short-circuit traces.** Traces inside an unevaluated branch of `&&` or `||` do not fire. If you expected a trace and saw none, check whether the left operand short-circuited.
- **`pub` matters for blueprints.** A type or constructor not marked `pub` will not appear in `plutus.json` — off-chain code-generators will not see it.

---

## 6. Stdlib anchor list

The names below are verified against `aiken-lang/stdlib` `main`. **Do not invent function names.** If the function you need is not on this list, either (a) it has a different name — search the stdlib repo, or (b) it does not exist — write it yourself.

### 6.1 Module index

```
aiken/cbor
aiken/collection/dict
aiken/collection/dict/strategy
aiken/collection/list
aiken/collection/pairs
aiken/crypto
aiken/crypto/bitwise
aiken/crypto/bls12_381
aiken/crypto/bls12_381/g1
aiken/crypto/bls12_381/g2
aiken/crypto/bls12_381/pairing
aiken/crypto/bls12_381/scalar
aiken/crypto/int224
aiken/crypto/int256
aiken/interval
aiken/math
aiken/math/rational
aiken/option
aiken/primitive/bytearray
aiken/primitive/int
aiken/primitive/string
cardano/address
cardano/address/credential
cardano/assets
cardano/assets/strategy
cardano/certificate
cardano/governance
cardano/governance/protocol_parameters
cardano/governance/voter
cardano/script_context
cardano/transaction
cardano/transaction/output_reference
cardano/transaction/script_purpose
```

### 6.2 `aiken/collection/list`

```
push, range, repeat
all, any, at, count, find, find_map, has, head, is_empty, index_of, last, length
expect_any, expect_at, expect_find, expect_find_map, expect_has, expect_head,
  expect_index_of, expect_last
delete, drop, drop_while, filter, filter_map, init, partition, slice, span, tail,
  take, take_while, unique
expect_delete, expect_drop, expect_init, expect_tail, expect_take
flat_map, for_each, indexed_map, map, map2, map3, reverse, sort, unzip
concat, difference, zip
foldl, foldl2, foldr, foldr2, indexed_foldr, reduce
```

### 6.3 `aiken/collection/dict`

```
empty                            // constant: Dict<k, v>
from_ascending_pairs, from_ascending_pairs_with, from_pairs, singleton
contains, find, get, get_or_else, has_key, is_empty, keys, size, values
expect_contains, expect_find, expect_get, expect_has_key
delete, difference_with, filter, insert, insert_with, map, pop
expect_delete, expect_pop, expect_tail
union, union_with
foldl, foldl2, foldr, foldr2
to_pairs
```

### 6.4 `aiken/interval`

Types: `Interval`, `IntervalBound`, `IntervalBoundType`.
Constants: `empty: Interval`, `everything: Interval`.

```
after, entirely_after, before, entirely_before, between, entirely_between
contains, is_empty, is_entirely_after, is_entirely_before
to_string, hull, includes, intersection
```

### 6.5 `aiken/crypto`

Types: `VerificationKey`, `VerificationKeyHash`, `Script`, `ScriptHash`, `Signature`, `DataHash`, `Hash<alg, a>`.

```
blake2b_224, blake2b_256, keccak_256, sha2_256, sha3_256
verify_ecdsa_signature, verify_ed25519_signature, verify_schnorr_signature
```

Use `blake2b_224` for credentials (28 bytes — `VerificationKeyHash`, `ScriptHash`, `PolicyId`). Use `blake2b_256` for transaction ids, datum hashes, and other 32-byte digests.

### 6.6 `aiken/option`

```
is_none, is_some, and_then, choice, flatten, map, map2, map3, or_try, or_else
```

### 6.7 `aiken/math`

```
abs, clamp, gcd, is_sqrt, log, log2, max, min, pow, pow2, sqrt
```

For non-integer math, use `aiken/math/rational` — there is no floating-point type.

### 6.8 `aiken/primitive/bytearray`

Type: `Byte`.

```
from_int_big_endian, from_int_little_endian, from_string
push, at, index_of, is_empty, length, test_bit
drop, slice, take, concat, compare
foldl, foldr, reduce
to_int_big_endian, to_int_little_endian, to_string, to_hex
starts_with, and_bytes, or_bytes, xor_bytes
```

### 6.9 `cardano/transaction`

Types: `TransactionId`, `ScriptPurpose`, `Transaction`, `ValidityRange`, `Input`, `OutputReference`, `Output`, `Datum`, `Redeemer`.

Functions:

```
find_input, resolve_input, find_datum, find_script_outputs
```

Constants: `placeholder: Transaction` (a default `Transaction` for tests — use with spread).

Key field names on `Transaction`: `inputs`, `reference_inputs`, `outputs`, `fee`, `mint`, `certificates`, `withdrawals`, `validity_range`, `extra_signatories`, `redeemers`, `datums`, `id`, `votes`, `proposal_procedures`, `current_treasury_amount`, `treasury_donation`.

### 6.10 `cardano/assets`

Types: `Lovelace`, `PolicyId`, `AssetName`, `Value` (opaque).
Constants: `ada_policy_id = ""`, `ada_asset_name = ""`, `zero: Value`.

```
from_asset, from_ascending_pairs, from_asset_list, from_lovelace
contains, has_any_nft, has_any_nft_strict, has_nft, has_nft_strict
is_zero, match, match_assets
lovelace_of, policies, quantity_of, tokens
expect_lovelace_of, expect_match, expect_match_assets, expect_quantity_of
negate, restricted_to, without_lovelace
add, difference, merge
flatten, flatten_with, reduce
to_dict, to_pairs
```

`Value` is opaque — manipulate it through these functions, do not destructure it.

### 6.11 `cardano/address`

Types: `Credential`, `Address`, `Referenced<a>`, `StakeCredential`, `PaymentCredential`.

```
from_script, from_verification_key, with_delegation_key, with_delegation_script
```

### 6.12 `cardano/certificate`, `cardano/governance`, `cardano/script_context`

`cardano/certificate` exposes `StakePoolId`, `Certificate`, `Delegate`, `DelegateRepresentative`.

`cardano/governance` exposes `ProposalProcedure`, `GovernanceAction`, `Vote`, `Voter`, `GovernanceActionId`, `ProtocolVersion`, `Constitution`, `Mandate`.

`cardano/script_context` exposes `ScriptContext`, `ScriptInfo` — used in the `else` fallback handler.

---

## Appendix A — Project structure and toolchain

### A.1 Project layout

`aiken new {org}/{repo}` produces:

```
.
├── README.md
├── aiken.toml
├── env
├── lib
│   └── {repo}
└── validators
    └── placeholder.ak
```

`lib/` is for library code (helper functions, types, shared logic). `validators/` is for top-level validator definitions. Tests can live in either directory.

### A.2 aiken.toml

```toml
name = "myorg/myproject"
version = "0.0.0"
compiler = "v1.1.21"
plutus = "v3"
license = "Apache-2.0"
description = "Aiken contracts for myorg/myproject"

[repository]
user = "myorg"
project = "myproject"
platform = "github"

[[dependencies]]
name = "aiken-lang/stdlib"
version = "v3.0.0"
source = "github"

[config]
```

- `compiler` pins the required compiler version.
- `plutus = "v3"` is the current default.
- `[[dependencies]]` lists github deps. Use `aiken packages add` rather than editing by hand.
- `[config.default]` exposes static constants — import them with `use config`.

### A.3 CLI workflow

| Command | Effect |
|---------|--------|
| `aiken fmt` | Format source files |
| `aiken check` | Type-check, resolve deps, run all tests. Preserves traces by default. |
| `aiken build` | Compile to UPLC. Emit `plutus.json` (CIP-0057 blueprint). Strips traces by default. |
| `aiken blueprint apply` | Apply parameters to a parameterised validator |
| `aiken blueprint address` | Compute the script address from the blueprint |
| `aiken docs` | Generate HTML docs |
| `aiken packages add ...` | Add a dependency |

Trace levels: `--trace-level silent|compact|verbose` on `build` and `check`. Mismatched levels between commands produce different script hashes — see §5.4.

### A.4 plutus.json (CIP-0057 blueprint)

`aiken build` emits a `plutus.json` at the project root. It contains the compiled validators (CBOR-encoded UPLC), their script hashes, and the JSON schema for each datum and redeemer. Off-chain code-generators consume this file to produce typed transaction builders.

A type or constructor must be `pub` to appear in the blueprint.

### A.5 Well-known packages

- `aiken-lang/prelude` — auto-imported base types and functions. You do not import this explicitly.
- `aiken-lang/stdlib` — `aiken/*` and `cardano/*` modules (§6).
- `aiken-lang/fuzz` — property-based testing with generators.
- `sidan-lab/vodka` — community helpers (e.g., `key_signed`, `valid_after`) and the `mocktail` test framework.

Community registry: https://packages.aiken-lang.org.

---

End of domain knowledge document.
