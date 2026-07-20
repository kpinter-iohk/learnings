# RESEARCH.md — Raw source material for Cardano/Aiken domain-knowledge primer

Compiled from authoritative sources on aiken-lang.org, the aiken-lang/stdlib repository, IOG's pogun-claude-meadhall cardano-mentor agent, and a community writeup on double-satisfaction. Each block is annotated with its source URL.

URL substitutions / 404 notes:
- `https://aiken-lang.org/example--vesting` 404s — vesting docs live at `/example--vesting/mesh`. The on-chain code is reproduced from `https://github.com/aiken-lang/site/blob/main/src/pages/example--vesting/mesh.mdx`.
- `https://aiken-lang.org/language-tour/pipelines` 404s — pipe operator documented under `/language-tour/control-flow`.
- `https://aiken-lang.org/getting-started` 404s — content at `/fundamentals/getting-started`.
- `https://raw.githubusercontent.com/input-output-hk/pogun-claude-meadhall/main/agents/cardano-mentor/agent.md` returns 404 to anonymous WebFetch but is accessible via `gh api repos/input-output-hk/pogun-claude-meadhall/contents/agents/cardano-mentor/agent.md` — content reproduced below.

---

## 1. eUTXO mental model

### (a) Contrast with account-based models

The Aiken docs do not directly compare to Ethereum, but the analogy they use is the post-it-note wall. Quoting verbatim:

> "An input is a reference to a previous output. Think of outputs as post-it notes with a unique serial number and inputs as being this serial number. A transaction is a document indicating which post-it notes should be destroyed and which new ones should be pinned to the wall."
>
> — https://aiken-lang.org/fundamentals/eutxo

> "An output that hasn't been spent yet (i.e. is still on the wall) is called — you guessed it — an _unspent transaction output_, or UTxO in short. The blockchain state results from looking at the entire wall of remaining post-it notes."
>
> — https://aiken-lang.org/fundamentals/eutxo

Account-based contrast (implicit): Cardano state is the set of UTxOs; there is no per-address mutable balance struct. Each input is consumed exactly once (uniqueness ensured by the protocol), so concurrent spends of the same UTxO are impossible by construction. Reward accounts are the one exception:

> "Cardano has introduced a restricted concept of account, similar to what exists on account-based ledgers. This account is, however, singular in many ways: It is defined by some stake credentials and owned by them; It can only receive rewards from the protocol but not from a user-defined transaction; It is automatically delegated."
>
> — https://aiken-lang.org/fundamentals/what-i-wish-i-knew

Concise distillation (paraphrased): eUTxO = stateless, deterministic, local; Ethereum = stateful, global mutable storage. Cardano validators are **predicates** evaluating a transaction; they cannot modify state — only approve/reject. A "state change" is modelled as consuming one UTxO and creating another with the new state in its datum.

### (b) The "validator returns Bool" framing

> "Scripts are like predicates. Said differently, they are functions that return a boolean value: `True` or `False`. To be considered valid, all scripts in a transaction must return `True`."
>
> — https://aiken-lang.org/fundamentals/eutxo

But the docs immediately disclaim this simplification:

> "Well, _not exactly_. We lied to you (but only a tiny bit). If we only had that, it would be hard to express more elaborate logic. In particular, capturing a state, which programs often require, would be infeasible. A state and transitions from that state. This is where the _extended UTxO_ model comes in. It adds two new components to what we've already seen: datums and redeemers."
>
> — https://aiken-lang.org/fundamentals/eutxo

In Aiken specifically, handlers are predicates:

> "Every handler is a predicate function: they must return `True` or `False`. When `True`, they authorize the action they are validating. Alternatively, to return false, they can instead halt using the `fail` keyword or via an invalid `expect` assignment."
>
> — https://aiken-lang.org/language-tour/validators

### (c) Role of datum, redeemer, script context

> "The datum is a free payload that developers can use to attach data to script execution. When a script is executed in a spending scenario, it receives not only the transaction as context but also the datum associated with the output being spent. The redeemer, on the other hand, is another piece of data that is also provided with the transaction for any script execution. Notice that the datum and redeemer intervene at two distinct moments. A datum is set when the output is created (i.e. when the post-it note is hung on the wall, it is part of the note). In contrast, the redeemer is provided only when spending the output (i.e. provided along with the form as it is handed over to the employee)."
>
> — https://aiken-lang.org/fundamentals/eutxo

Parametric-function analogy:

```console
             Script
          ╭─────────╮
    f(x) = x * a + b  = true | false
           ╿   ╿   ╿
  Redeemer ┘   │   │
               └─┬─┘
               Datum
```

> "The script defines the function as a whole. … The datum corresponds to the parameters of the function. It allows _configuring the function_. … the function argument … is the redeemer (as well as the rest of the transaction)."
>
> — https://aiken-lang.org/fundamentals/eutxo

Script context: the third component is the **ScriptContext** (the transaction plus a `ScriptInfo` that says which purpose/datum/redeemer the current invocation is for). In Aiken handlers, the context is passed as the `self: Transaction` parameter, plus a per-purpose target (PolicyId, OutputReference, Credential, Certificate, Voter, ProposalProcedure). The fallback `else(ctx: ScriptContext)` receives the full untyped context.

> "If we take a step back and look at the typical public/private key procedure for spending funds, we can see how eUTxO is merely a generalization of that. Indeed, the public key (hash) can be seen as _the datum_, whereas the signature is the _redeemer_. The script is the digital signature verification algorithm that controls whether the signature is valid w.r.t. the provided key."
>
> — https://aiken-lang.org/fundamentals/eutxo

### (d) Spending vs reference inputs, and the six script purposes

> "Each purpose indicates _for what purpose_ a script is being executed. During validation, that information is passed to the script alongside the transaction and the redeemer. **Note that only scripts executed with the `spend` purpose are given a datum.** This is because they can leverage the data payload present in outputs, unlike the other purposes that do not get this opportunity."
>
> — https://aiken-lang.org/fundamentals/eutxo (emphasis added; this is the LLM gotcha)

The six purposes (from the same page):

- `mint` — controls how to mint or burn assets;
- `spend` — controls how to spend outputs;
- `withdraw` — controls how to withdraw consensus rewards;
- `publish` — controls how to publish delegation certificates;
- `vote` — controls how to vote on proposal procedures;
- `propose` — controls constitutionality of proposal procedures (constitution guardrails — only one such script in the entire ledger).

Reference inputs: the eutxo.mdx page does **not** explicitly describe reference inputs; the concept appears in the Common Design Patterns page implicitly. From the STT/state-machine pattern:

> "It is often useful to have a mutable state which either changes with each transaction, or on a periodic basis. One way to ensure that a datum is not 'spoofed' is to ensure that the input or reference input with that datum contains an NFT which has been generated to be unique."
>
> — https://aiken-lang.org/fundamentals/common-design-patterns

Reference inputs (CIP-31, post-Vasil): a transaction can attach a UTxO as a **reference input** — its datum/value are visible to validators but the UTxO is **not consumed**. This is how oracles, parameter dictionaries, and shared scripts (CIP-33) are distributed without forcing recreation each block. In `cardano/transaction.Transaction`, both `inputs` and `reference_inputs` are fields (see stdlib `cardano/transaction.ak` — `Transaction` type definition contains both).

### (e) IOG cardano-mentor agent: review priorities

From `agents/cardano-mentor/agent.md` (verbatim excerpts):

> "You are a Principal Cardano Protocol Engineer with deep expertise in Aiken smart contracts, PlutusV3, the eUTxO model, beacon-gated state machines, and Rust transaction builders for Cardano. You review Cardano-specific code with zero tolerance for on-chain bugs — validator mistakes can lock or drain funds."

Core review concerns:
1. On-chain correctness — validator bugs can lock or drain funds
2. On-chain/off-chain consistency — Rust builders must produce transactions that validators accept
3. Beacon integrity — only the correct minting policy can mint/burn beacon tokens
4. Datum continuity — immutable fields preserved exactly when UTxOs are consumed and recreated
5. Authorization — correct credential verified for each operation

Aiken-validator checklist:
- **Authorization checks**: Is the correct credential verified? (staking credential, token presence, signature)
- **Datum continuity**: When consuming and recreating a UTxO, are immutable fields preserved exactly? Are mutable fields constrained correctly?
- **Beacon accounting**: Are the right beacons minted/burned? Do quantities match?
- **Value preservation**: Is ADA/token value conserved correctly? Are minUTxO requirements met?
- **Boundary conditions**: Integer overflow in fraction arithmetic? Off-by-one in epoch calculations? Edge cases with zero values?
- **Redeemer dispatch**: Does the spending validator handle all redeemers? Are there unreachable paths?

Cross-layer consistency checks (Aiken ↔ Rust builder):
- Datum field order matches between Aiken types and Rust PlutusData encoding
- Beacon calculation logic is identical (e.g., `blake2b_256(tx_id ++ output_index)`)
- Redeemer encoding matches
- Fee aggregation in builder matches validator expectations

Terminology discipline:
- "eUTxO" — not "UTXO" when referring to Cardano's extended model
- "Datum" — the on-chain data attached to a UTxO
- "Redeemer" — the data provided when spending a UTxO
- "Minting policy" — not "mint contract" or "token contract"
- "Spending validator" — not "smart contract" (be specific about which validator type)

— Source: `gh api repos/input-output-hk/pogun-claude-meadhall/contents/agents/cardano-mentor/agent.md`

---

## 2. Aiken syntax cheatsheet (distinct from Plutus / other languages)

### Primitive types

| Type | Literal syntax |
|---|---|
| `Bool` | `True`, `False` |
| `Int` | `42`, `1_000_000`, `0xF`, `0b1111`, `0o17` (arbitrary-precision integer) |
| `ByteArray` | `#[10, 255]` (byte list), `"foo"` (utf-8 → bytes), `#"666f6f"` (hex) |
| `String` | `@"Hello, Aiken!"` (note the leading `@` — strings are for tracing only) |
| `List<a>` | `[1, 2, 3]`, `[1, ..[2, 3]]` (cons) |
| `Tuple` | `(10, "hello")`, `(1, 4, [0])` |
| `Pair` | `Pair(14, "aiken")` |
| `Option<a>` | `Some(a)`, `None` |
| `Ordering` | `Less`, `Equal`, `Greater` |
| `Void` | `Void` |
| `Data` | opaque, any serialisable value |
| `Never` | identical to `None` |

Notes: "Inserting at the front of a list is very fast"; all data structures are immutable; "Strings are primarily for debugging/tracing, whereas ByteArray handles actual data."

— Source: https://aiken-lang.org/language-tour/primitive-types

### Custom types (records and ADTs)

```aiken
// Record with named fields (single-constructor shorthand)
type Datum {
  signer: ByteArray,
  count: Int,
}

// Record with unnamed positional fields
type DatumNameless {
  DatumNameless(ByteArray, Int)
}

// Sum type (ADT) with multiple constructors
type User {
  LoggedIn { username: ByteArray }
  Guest
}

// Generic
type Box<inner_type> {
  Box(inner: inner_type)
}

// Type alias
type MyNumber = Int
type Person = (String, Int)
```

Pattern matching is exhaustive:

```aiken
fn get_name(user: User) -> ByteArray {
  when user is {
    LoggedIn { username } -> username
    Guest -> "Guest user"
  }
}
```

Patterns: `_` wildcard, `|` alternation, `..` spread (ignore remaining fields), `[a, ..]` list-head/tail, nested patterns like `Some(Dog { name, .. })`.

Field access via dot notation on single-constructor records: `dog.name`. Record-update syntax: `Person { ..person, age: person.age + 1 }`.

Custom encoding annotations:
```aiken
// Encode bool as PlutusData with explicit tags
type Bool {
  @tag(1)
  True
  @tag(0)
  False
}

// Encode as PlutusList instead of Constr
@list
type Datum {
  signer: ByteArray,
  count: Int,
}
```

Upcasting (custom-type → Data) is implicit and safe. Downcasting (Data → custom-type) requires `expect` or `if..is`.

— Source: https://aiken-lang.org/language-tour/custom-types

### Functions

```aiken
// Module-local (private)
fn add(x: Int, y: Int) -> Int {
  x + y
}

// Exported
pub fn identity(x: a) -> a { x }

// Anonymous
let add = fn(x, y) { x + y }
add(1, 2)

// Function capture (partial application)
let add_one = add(1, _)

// Labelled arguments
fn replace(self: String, pattern: String, replacement: String) { }
replace(pattern: @",", replacement: @" ", self: @"A,B,C")

// Generic
fn list_of_two(v: a) -> List<a> { [v, v] }
```

`pub fn` exports; bare `fn` is module-private. Recursive anonymous functions are not supported — use top-level definitions.

— Source: https://aiken-lang.org/language-tour/functions

### Control flow

`when..is` (exhaustive pattern matching):
```aiken
fn length(xs: MyList<a>) -> Int {
  when xs is {
    Empty -> 0
    Prepend(_head, tail) -> 1 + length(tail)
  }
}
```

`expect` (downcast / non-exhaustive match; halts on failure):
```aiken
expect Some(y) = x
expect my_datum: MyDatum = data
```

`if..is` (soft cast, no halt — returns Bool):
```aiken
if d is Foo {
  d.foo == 1
} else if d is Bazz(y): Bar {
  y == 1
} else {
  False
}
```

`if`/`else`:
```aiken
if some_bool { "It's true!" } else { "It's not true." }
```

Pipe `|>`:
```aiken
string
  |> string_builder.from_string
  |> string_builder.reverse
  |> string_builder.to_string
// "Each line applies the function to the result of the previous line."

// Combines with capture:
1 |> add(3) |> add(6) |> add(9)
```

`trace` and the `?` "trace-if-false" operator:
```aiken
trace @"redeemer": string.from_bytearray(redeemer.msg)
must_say_hello? && must_be_signed?  // trace only when the marked sub-expression is False
```

`/// doc-comments` on `expect` produce custom runtime traces:
```aiken
/// life, universe and everything.
expect answer == 42
```

`fail` and `todo`:
```aiken
fail @"reason"   // halts execution; no compilation warning
todo @"note"     // halts; produces compilation warning
```

— Sources: https://aiken-lang.org/language-tour/control-flow, https://aiken-lang.org/language-tour/troubleshooting

### Modules and imports

Modules are organised by file path. A file at `lib/straw_hats/sunny.ak` is the module `straw_hats/sunny`. `pub` exports.

```aiken
// Qualified
use straw_hats/sunny
pub fn go() { sunny.set_sail() }

// Alias
use animal/dog as kitty

// Unqualified imports
use animal/dog.{Dog, stroke}

// Unqualified + alias
use animal/dog.{Dog, stroke} as kitty
```

Notable specifics:
- `pub opaque type` — exports the type name but hides constructors/fields.
- The `aiken` prelude is automatically available.
- `aiken/builtin` exposes raw Plutus core builtins.
- `env/` directory holds environment modules; reference as `use env` regardless of filename.
- `[config.default]` block in `aiken.toml` exposes static values; import via `use config`.

— Source: https://aiken-lang.org/language-tour/modules

---

## 3. Validator structure and purposes

### The `validator name { ... }` block

> "In Aiken, you can promote some functions to _validator handlers_ using the keyword `validator`. … a validator is a named block that contains one or more handlers. The handler name must match Cardano's well-known purposes: `mint`, `spend`, `withdraw`, `publish`, `vote` or `propose`."
>
> — https://aiken-lang.org/language-tour/validators

### Handler signatures (the LLM gotcha — only `spend` gets a datum)

| Purpose | Target type | Signature |
|---|---|---|
| `mint` | `PolicyId` | `mint(redeemer: R, policy_id: PolicyId, self: Transaction)` |
| `spend` | `OutputReference` | `spend(datum: Option<D>, redeemer: R, utxo: OutputReference, self: Transaction)` |
| `withdraw` | `Credential` | `withdraw(redeemer: R, account: Credential, self: Transaction)` |
| `publish` | `Certificate` | `publish(redeemer: R, certificate: Certificate, self: Transaction)` |
| `vote` | `Voter` | `vote(redeemer: R, voter: Voter, self: Transaction)` |
| `propose` | `ProposalProcedure` | `propose(redeemer: R, proposal: ProposalProcedure, self: Transaction)` |

> "With the exception of the `spend` handler, each handler is a function with exactly three arguments: A **redeemer**, … A **target**, … and A **transaction**. … The `spend` handler takes an additional first argument which is an optional datum"
>
> — https://aiken-lang.org/language-tour/validators

Critical: `datum` in `spend` is `Option<D>`, never `D`. The docs explain why:

> "Because there's no way to enforce that the datum is present (you cannot prevent anyone from sending/locking assets to/in your validator), it always produces an `Option<T>`. Nevertheless, should your contract require a datum to be present, then it is straightforward to enforce this constraint using `expect`."
>
> — https://aiken-lang.org/language-tour/validators

Canonical pattern:
```aiken
validator my_script {
  spend(datum_opt: Option<MyDatum>, redeemer: MyRedeemer, input: OutputReference, self: Transaction) {
    expect Some(datum) = datum_opt
    // ... logic ...
  }
}
```

### Multi-handler validators (one validator, multiple purposes)

```aiken
use cardano/address.{Credential}
use cardano/assets.{PolicyId}
use cardano/certificate.{Certificate}
use cardano/governance.{ProposalProcedure, Voter}
use cardano/transaction.{Transaction, OutputReference}

validator my_script {
  mint(redeemer: MyMintRedeemer, policy_id: PolicyId, self: Transaction) { todo }
  spend(datum: Option<MyDatum>, redeemer: MySpendRedeemer, utxo: OutputReference, self: Transaction) { todo }
  withdraw(redeemer: MyWithdrawRedeemer, account: Credential, self: Transaction) { todo }
  publish(redeemer: MyPublishRedeemer, certificate: Certificate, self: Transaction) { todo }
  vote(redeemer: MyVoteRedeemer, voter: Voter, self: Transaction) { todo }
  propose(redeemer: MyProposeRedeemer, proposal: ProposalProcedure, self: Transaction) { todo }
}
```

All handlers in one `validator` block share the same script hash (and therefore the same address / policy id). That property is what enables withdraw-zero / forwarding-validation patterns.

### Fallback `else` handler

> "A special handler can thus serve as a fallback / catch-all with one notable difference: the fallback handler takes a single argument of type `ScriptContext`. It is then your responsibility as a smart contract developer to assert the script purposes and recover your redeemer and/or datum."
>
> — https://aiken-lang.org/language-tour/validators

```aiken
validator my_multi_purpose_script {
  mint(redeemer: MyRedeemer, policy_id: PolicyId, self: Transaction) { todo }
  spend(datum_opt: Option<MyDatum>, redeemer: MyRedeemer, input: OutputReference, self: Transaction) {
    expect Some(datum) = datum_opt
    todo
  }
  else(_ctx: ScriptContext) {
    fail @"unsupported purpose"
  }
}
```

Default when no fallback is specified: always-rejecting.

### Validator parameters

> "Validators themselves can take _parameters_, which represent configuration elements that must be provided to create an instance of the validator. Once provided, parameters are embedded within the compiled validator and part of the generated code. Hence they must be provided before any address can be calculated for the corresponding validator."
>
> — https://aiken-lang.org/language-tour/validators

```aiken
validator my_script(utxo_ref: OutputReference) {
  mint(redeemer: Data, policy_id: PolicyId, self: Transaction) {
    expect list.any(self.inputs, fn(input) { input.output_reference == utxo_ref })
    todo
  }
}
```

### Calling handlers as functions (for testing)

```aiken
test return_true_when_utxo_ref_match() {
  let utxo_ref = todo
  let redeemer = todo
  let policy_id = todo
  let transaction = todo
  my_script.mint(utxo_ref, redeemer, policy_id, transaction)
  //         ^^ parameters precede handler args
}
```

— Source: https://aiken-lang.org/language-tour/validators

---

## 4. Stdlib anchor list (anti-hallucination whitelist)

### All modules in `aiken-lang/stdlib` (as of `main`, verified via `gh api repos/aiken-lang/stdlib/git/trees/main?recursive=1`)

```
aiken/cbor
aiken/collection
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

— Source: https://github.com/aiken-lang/stdlib (lib/ tree at main)

### Most-used modules — exhaustive function/type lists

#### `aiken/collection/list`

Types/aliases: (none of note)

Functions:
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

— Source: stdlib `lib/aiken/collection/list.ak`

#### `aiken/collection/dict`

Constants: `empty: Dict<key, value>`

Functions:
```
from_ascending_pairs, from_ascending_pairs_with, from_pairs, singleton
expect_contains, expect_find, expect_get, expect_has_key
contains, find, get, get_or_else, has_key, is_empty, keys, size, values
delete, difference_with, filter, insert, insert_with, map, pop
expect_delete, expect_pop, expect_tail
union, union_with
foldl, foldl2, foldr, foldr2
to_pairs
```

— Source: stdlib `lib/aiken/collection/dict.ak`

#### `aiken/interval`

Types: `Interval`, `IntervalBound`, `IntervalBoundType`

Functions:
```
after, entirely_after, before, entirely_before, between, entirely_between
contains, is_empty, is_entirely_after, is_entirely_before
to_string, hull, includes, intersection
```

Constants: `empty: Interval`, `everything: Interval`

— Source: stdlib `lib/aiken/interval.ak`

#### `aiken/crypto`

Types: `VerificationKey`, `VerificationKeyHash`, `Script`, `ScriptHash`, `Signature`, `DataHash`, `Hash<alg, a>`

Functions:
```
blake2b_224, blake2b_256, keccak_256, sha2_256, sha3_256
verify_ecdsa_signature, verify_ed25519_signature, verify_schnorr_signature
```

— Source: stdlib `lib/aiken/crypto.ak`

#### `aiken/option`

Functions:
```
is_none, is_some, and_then, choice, flatten, map, map2, map3, or_try, or_else
```

— Source: stdlib `lib/aiken/option.ak`

#### `aiken/math`

Functions: `abs, clamp, gcd, is_sqrt, log, log2, max, min, pow, pow2, sqrt`

— Source: stdlib `lib/aiken/math.ak`

#### `aiken/primitive/bytearray`

Type: `Byte`

Functions:
```
from_int_big_endian, from_int_little_endian, from_string
push, at, index_of, is_empty, length, test_bit
drop, slice, take, concat, compare
foldl, foldr, reduce
to_int_big_endian, to_int_little_endian, to_string, to_hex
starts_with, and_bytes, or_bytes, xor_bytes
```

— Source: stdlib `lib/aiken/primitive/bytearray.ak`

#### `cardano/transaction`

Types: `TransactionId`, `ScriptPurpose`, `Transaction`, `ValidityRange`, `Input`, `OutputReference`, `Output`, `Datum`, `Redeemer`

Functions:
```
find_input, resolve_input, find_datum, find_script_outputs
```

Constants: `placeholder: Transaction` (test helper)

Key field names on `Transaction`: `inputs`, `reference_inputs`, `outputs`, `fee`, `mint`, `certificates`, `withdrawals`, `validity_range`, `extra_signatories`, `redeemers`, `datums`, `id`, `votes`, `proposal_procedures`, `current_treasury_amount`, `treasury_donation`. (Confirm exact field list against `lib/cardano/transaction.ak` — these are the standard Conway-era fields.)

— Source: stdlib `lib/cardano/transaction.ak`

#### `cardano/assets`

Types: `Lovelace`, `PolicyId`, `AssetName`, `Value` (opaque)

Constants: `ada_policy_id = ""`, `ada_asset_name = ""`, `zero: Value`

Functions:
```
from_asset, from_ascending_pairs, from_asset_list, from_lovelace
contains, has_any_nft, has_any_nft_strict, has_nft, has_nft_strict
is_zero, match, match_assets
lovelace_of, policies, quantity_of, tokens
expect_lovelace_of, expect_match, expect_match_assets, expect_quantity_of
negate, restricted_to, without_lovelace, expect_tail
add, difference, merge
flatten, flatten_with, reduce
to_dict, to_pairs
```

— Source: stdlib `lib/cardano/assets.ak`

#### `cardano/address`

Types: `Credential`, `Address`, `Referenced<a>`, `StakeCredential`, `PaymentCredential`

Functions: `from_script`, `from_verification_key`, `with_delegation_key`, `with_delegation_script`

— Source: stdlib `lib/cardano/address.ak`

#### `cardano/certificate`

Types: `StakePoolId`, `Certificate`, `Delegate`, `DelegateRepresentative`

— Source: stdlib `lib/cardano/certificate.ak`

#### `cardano/governance`

Types: `ProposalProcedure`, `GovernanceAction`, `Vote`, `TransactionId`, `GovernanceActionId`, `ProtocolVersion`, `Constitution`, `Mandate`, `Voter`

— Source: stdlib `lib/cardano/governance.ak`

#### `cardano/script_context`

Types: `ScriptContext`, `ScriptInfo`

— Source: stdlib `lib/cardano/script_context.ak`

---

## 5. Anti-pattern catalog

### 5.1 Double Satisfaction

**(a) Name:** Double Satisfaction
**(b) Mistake:** A validator predicates spending on a condition (e.g., "at least N ADA paid to Bob") that can be satisfied once by a single output, while multiple input UTxOs all rely on that same output.
**(c) Why wrong:** The validator runs once per input but inspects the whole transaction. If two UTxOs at the same address are spent in one tx, and each validator only checks "Bob received ≥ N ADA," a single payment satisfies both.
**(d) Fix:** **Tag outputs** with a value unique to the input — embed the input's `OutputReference` in the output's datum and require a 1-to-1 match.

Verbatim Aiken docs example of the vulnerable pattern:

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

Fix using tagged outputs (`own_ref` embedded as the output datum):

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

Vacuumlabs framing of the same bug:

> "Each contract's validator validates the transaction independently and they all must be satisfied for the transaction to be validated by the blockchain."
>
> "Eve exploits this by spending both contracts in a single transaction while paying only 120 ADA total (instead of 200 ADA). Both validators see the same output—120 ADA going to Alice—and both pass validation independently."

— Sources: https://aiken-lang.org/fundamentals/common-design-patterns and https://medium.com/@vacuumlabs_auditing/cardano-vulnerabilities-1-double-satisfaction-219f1bc9665e

### 5.2 Opaque-Data misuse (unchecked downcast)

**(a) Name:** Opaque-Data misuse / unsafe downcast
**(b) Mistake:** Treating an arbitrary `Data` value as a specific type without going through `expect` or `if..is`.
**(c) Why wrong:** Upcasting (custom → Data) is implicit and safe; **downcasting (Data → custom) can fail**. Misuse silently accepts or rejects malformed datums.
**(d) Fix:** Use `expect my_datum: MyDatum = data` to halt on bad shape; use `if data is MyDatum` for soft-cast Bool semantics. In the tagged-output example above, the docs explicitly use a soft cast (not `expect`) on a foreign output's datum because "the transaction might still contain other kind of outputs that we simply chose to ignore. Using `expect` here would cause the entire transaction to be rejected for any output that doesn't have a datum of that particular shape."

— Source: https://aiken-lang.org/language-tour/custom-types and https://aiken-lang.org/fundamentals/common-design-patterns

### 5.3 Validator-purpose / datum confusion

**(a) Name:** Datum expected in non-spend handler
**(b) Mistake:** Writing a `mint`, `withdraw`, `publish`, `vote`, or `propose` handler that tries to receive or use a datum.
**(c) Why wrong:** Only `spend` handlers receive a datum (and only as `Option<T>`). The other five purposes operate on credentials/policies/certificates/voters/proposals — they have no associated UTxO datum.
**(d) Fix:** Move datum-dependent state into the `spend` handler. If a `mint` policy needs to check a datum, it must locate the relevant input/reference-input within `self.inputs` or `self.reference_inputs` and inspect that input's `output.datum`.

> "**Note that only scripts executed with the `spend` purpose are given a datum.**"
>
> — https://aiken-lang.org/fundamentals/eutxo

> "The datum is always `Option<T>` in spend handlers since you can't prevent someone from sending assets without a datum."
>
> — https://aiken-lang.org/faq Q9

### 5.4 Trace in production changes hash

**(a) Name:** Trace-level drift between check and build
**(b) Mistake:** Leaving traces enabled in `aiken build` — or comparing addresses computed under different trace levels.
**(c) Why wrong:** "While enabling or disabling traces doesn't change the semantic of your program, it effectively changes its hash value, and thus its associated addresses."
**(d) Fix:** Default `aiken build` strips traces; `aiken check` keeps them. Use `--trace-level silent` for hash parity between dev and prod; never rely on an address computed with traces. For production debugging at known cost, use `--trace-level compact` (preserves only the label before `:`).

— Source: https://aiken-lang.org/language-tour/troubleshooting

### 5.5 Datum continuity violation

**(a) Name:** Datum continuity break (mutable/immutable field confusion)
**(b) Mistake:** When a UTxO holding state is consumed and recreated, the recreated output's datum either drops an immutable field, mutates one that should be invariant, or fails to constrain a mutable one (e.g., counter must increment by exactly 1).
**(c) Why wrong:** State machines on eUTxO are encoded as input-datum → output-datum transitions. A break lets an attacker forge state.
**(d) Fix:** Explicit `==`/`+1`/`merge` checks on every field of the continuing datum. The STT counter example below shows the pattern. From cardano-mentor: "When consuming and recreating a UTxO, are immutable fields preserved exactly? Are mutable fields constrained correctly?"

— Source: pogun-claude-meadhall cardano-mentor agent.md

### 5.6 Beacon / STT accounting

**(a) Name:** Beacon/STT spoofing
**(b) Mistake:** Relying on an input's datum without proving the input carries a uniqueness-anchoring NFT.
**(c) Why wrong:** Anyone can create a UTxO with arbitrary datum at any address. Without an STT (State Thread Token — a one-shot-minted NFT), the validator might accept a forged input.
**(d) Fix:** Mint an NFT under a one-shot policy parameterized by an `OutputReference`. Require all state-bearing inputs/outputs to carry that NFT. Verify both `mint` quantity (== 1) and that the parameter UTxO was actually consumed.

Verbatim STT pattern:

```aiken
validator counter_stt(utxo_ref: OutputReference) {
  mint(_redeemer: Data, policy_id: PolicyId, self: Transaction) {
    let Transaction { inputs, outputs, mint, .. } = self
    expect [Pair(_asset_name, quantity)] =
      mint |> assets.tokens(policy_id) |> dict.to_pairs()
    let is_output_consumed =
      list.any(inputs, fn(input) { input.output_reference == utxo_ref })
    expect Some(nft_output) =
      list.find(outputs, fn(output) { list.has(policies(output.value), policy_id) })
    expect InlineDatum(datum) = nft_output.datum
    expect counter: Int = datum
    is_output_consumed? && (1 == quantity)? && counter == 0
  }

  spend(_optional_datum: Option<Data>, _redeemer: Data, own_ref: OutputReference, self: Transaction) {
    let Transaction { inputs, outputs, .. } = self
    expect Some(own_input) = list.find(inputs, fn(input) { input.output_reference == own_ref })
    expect Script(own_script_hash) = own_input.output.address.payment_credential
    let is_signed_by_operator = list.has(self.extra_signatories, config.operator)
    expect Some(stt_input) =
      list.find(inputs, fn(input) { list.has(policies(input.output.value), own_script_hash) })
    expect InlineDatum(input_datum) = stt_input.output.datum
    expect counter_input: Int = input_datum
    expect Some(stt_output) =
      list.find(outputs, fn(output) { list.has(policies(output.value), own_script_hash) })
    expect InlineDatum(output_datum) = stt_output.datum
    expect counter_output: Int = output_datum
    expect stt_input.output.address == stt_output.address
    is_signed_by_operator? && (counter_output == counter_input + 1)?
  }
}
```

— Source: https://aiken-lang.org/fundamentals/common-design-patterns

### 5.7 On-chain / off-chain encoding drift

**(a) Name:** PlutusData encoding mismatch
**(b) Mistake:** Off-chain transaction builder (e.g., Rust, TypeScript) encodes a datum/redeemer whose field order, optional wrapping, or type widths disagree with the Aiken definition.
**(c) Why wrong:** The validator decodes the on-chain `Data` against its declared type. A field-order swap or missing `Option` wrapper causes `expect` to fail (or worse — silently parses into the wrong constructor).
**(d) Fix:** Treat the Aiken type as the canonical schema. Cross-check via the Plutus blueprint `plutus.json` (CIP-0057) generated by `aiken build`. cardano-mentor explicitly lists this as a HIGH-severity cross-layer concern: "Datum field order matches between Aiken types and Rust PlutusData encoding."

Aiken offers explicit annotations to control encoding: `@tag(n)` on a constructor for stable indices; `@list` on the type to force `PlutusList` instead of `Constr`.

— Sources: https://aiken-lang.org/language-tour/custom-types and pogun cardano-mentor

### 5.8 Integer arithmetic (no floating point)

**(a) Name:** Floating-point arithmetic
**(b) Mistake:** Assuming Aiken has `Float` or decimals.
**(c) Why wrong:** Aiken has **no floating-point type**. `Int` is arbitrary-precision. Rational math goes through `aiken/math/rational`. Naive percent/fraction code without rationals silently truncates.
**(d) Fix:** Use integer arithmetic with explicit scaling (e.g., basis points × 10000), or use `aiken/math/rational` for exact fractions. cardano-mentor flags "integer overflow in fraction arithmetic" as a boundary-condition concern.

— Sources: https://aiken-lang.org/language-tour/primitive-types and pogun cardano-mentor

### 5.9 Narrow validity intervals

**(a) Name:** Over-tight validity range
**(b) Mistake:** Setting the transaction validity interval to a 1-2 slot window.
**(c) Why wrong:** "Blocks are produced every ~20 seconds on average," so extremely tight intervals increase missing-block risk.
**(d) Fix:** Use wider intervals (tens of seconds to minutes) unless you have a specific reason. Remember validators see the bound, not the exact slot — a lower bound `A` only tells you "current time ≥ A".

> "Note that because we don't control the upper-bound, it could very much be that this transaction is executed 30 years after the vesting delay. Yet, from the perspective of the vesting script, this is perfectly okay."
>
> — https://aiken-lang.org/example--vesting/mesh

— Source: https://aiken-lang.org/faq Q32

### 5.10 Reserialization drift

**(a) Name:** Reserialize-then-hash mismatch
**(b) Mistake:** Off-chain code deserializes a tx/datum and re-serializes it before hashing.
**(c) Why wrong:** "There's no canonical serialization of objects on Cardano. … the recommended strategy when dealing with deserialized objects that need to be reserialized is to always preserve the original bytes and not attempt to reserialize anything."
**(d) Fix:** Carry original bytes through your pipeline; only hash the bytes you actually observed on-chain.

— Source: https://aiken-lang.org/fundamentals/what-i-wish-i-knew

### 5.11 Policy-id hashing forgets the language tag

**(a) Name:** Policy-id miscalculation
**(b) Mistake:** Computing a policy id as `blake2b_224(serialized_script)` without the language discriminator byte.
**(c) Why wrong:** "Raw scripts aren't exact pre-image of their hash digest. Before hashing, scripts are prefixed with a certain discriminator byte depending on the language."
- `Native` → `0x00`
- `Plutus V1` → `0x01`
- `Plutus V2` → `0x02`
- `Plutus V3` → `0x03`
**(d) Fix:** Always prefix with the correct version byte before `blake2b_224`. (Note: `aiken build` produces correct hashes in `plutus.json` — issues arise only when computing hashes off-chain manually.)

— Source: https://aiken-lang.org/fundamentals/what-i-wish-i-knew

### 5.12 Forwarding-validation misuse

**(a) Name:** Per-input revalidation (missed optimization, but also correctness)
**(b) Mistake:** Running heavy authorization logic in `spend` for every input separately.
**(c) Why wrong:** "Running identical logic across multiple inputs incurs high execution costs. Delegation to withdrawal scripts optimizes budgets significantly." Beyond cost, copy-pasted logic invites copy-paste bugs.
**(d) Fix:** Use the withdraw-zero pattern: validators ensure the *withdraw* handler is invoked (with 0 lovelace) and forward all business logic there. The withdraw handler runs **once per tx** regardless of how many UTxOs are spent.

> "By enforcing withdrawals from a specific given script, we can effectively 'forward' the validation to this script being evaluated with the `withdraw` script purpose. This is possible in particular because it is always possible to withdraw an amount of 0 lovelace."
>
> — https://aiken-lang.org/fundamentals/common-design-patterns

### 5.13 Byron addresses leak into Plutus context

**(a) Name:** Byron address in script context
**(b) Mistake:** Off-chain logic constructs a Plutus tx that includes Byron-address inputs/outputs.
**(c) Why wrong:** "Outputs locked by a Byron address or inputs corresponding to such outputs are **forbidden** in transactions that have Plutus scripts!"
**(d) Fix:** Filter Byron addresses out before tx construction; treat them as deprecated.

— Source: https://aiken-lang.org/fundamentals/what-i-wish-i-knew

### 5.14 Misc smaller FAQ traps

- **Aiken ≠ Rust:** "Aiken is its own language — its compiler simply happens to be written in Rust." Don't reach for Rust idioms.
- **Pattern matches must be exhaustive** — the compiler enforces all variants.
- **`expect` vs `if..is`:** `expect` halts on mismatch; `if..is` returns `False`. Choose deliberately.
- **`fail` vs `todo`:** Both halt; `todo` emits a compile-time warning (intentional placeholder).
- **Short-circuit traces:** "Only the trace `is_even` will be captured, because `is_odd` is in fact never evaluated."

— Source: https://aiken-lang.org/faq, https://aiken-lang.org/language-tour/troubleshooting

---

## 6. Worked example seed: vesting validator (complete)

From `https://aiken-lang.org/example--vesting/mesh` (canonical source: aiken-lang/site, `src/pages/example--vesting/mesh.mdx`).

The contract: funds are locked with a `lock_until` timestamp, an `owner` (who can always reclaim) and a `beneficiary` (who can claim only after `lock_until`).

### Datum

```aiken
use aiken/crypto.{VerificationKeyHash}

pub type VestingDatum {
  /// POSIX time in milliseconds, e.g. 1672843961000
  lock_until: Int,
  /// Owner's credentials
  owner: VerificationKeyHash,
  /// Beneficiary's credentials
  beneficiary: VerificationKeyHash,
}
```

### aiken.toml dependency

```toml
[[dependencies]]
name = "sidan-lab/vodka"
version = "0.1.1-beta"
source = "github"
```

(Vodka exposes `key_signed` and `valid_after` as helper functions over `extra_signatories` and `validity_range`.)

### Full validator

```aiken
use cardano/transaction.{OutputReference, Transaction}
use vodka_extra_signatories.{key_signed}
use vodka_validity_range.{valid_after}
use aiken/crypto.{VerificationKeyHash}

pub type VestingDatum {
  /// POSIX time in milliseconds, e.g. 1672843961000
  lock_until: Int,
  /// Owner's credentials
  owner: VerificationKeyHash,
  /// Beneficiary's credentials
  beneficiary: VerificationKeyHash,
}

validator vesting {
  // In principle, scripts can be used for different purpose (e.g. minting
  // assets). Here we make sure it's only used when 'spending' from a eUTxO
  spend(
    datum_opt: Option<VestingDatum>,
    _redeemer: Data,
    _input: OutputReference,
    tx: Transaction,
  ) {
    expect Some(datum) = datum_opt
    or {
      key_signed(tx.extra_signatories, datum.owner),
      and {
        key_signed(tx.extra_signatories, datum.beneficiary),
        valid_after(tx.validity_range, datum.lock_until),
      },
    }
  }

  else(_) {
    fail
  }
}
```

### Test (uses vodka's mocktail)

```aiken
use mocktail.{complete, invalid_before, mocktail_tx, required_signer_hash}
use mocktail/virgin_key_hash.{mock_pub_key_hash}
use mocktail/virgin_output_reference.{mock_utxo_ref}

type TestCase {
  is_owner_signed: Bool,
  is_beneficiary_signed: Bool,
  is_lock_time_passed: Bool,
}

fn get_test_tx(test_case: TestCase) {
  let TestCase { is_owner_signed, is_beneficiary_signed, is_lock_time_passed } =
    test_case
  mocktail_tx()
    |> required_signer_hash(is_owner_signed, mock_pub_key_hash(1))
    |> required_signer_hash(is_beneficiary_signed, mock_pub_key_hash(2))
    |> invalid_before(is_lock_time_passed, 1672843961001)
    |> complete()
}

fn vesting_datum() {
  VestingDatum {
    lock_until: 1672843961000,
    owner: mock_pub_key_hash(1),
    beneficiary: mock_pub_key_hash(2),
  }
}

test success_unlocking() {
  let output_reference = mock_utxo_ref(0, 1)
  let datum = Some(vesting_datum())
  let test_case =
    TestCase {
      is_owner_signed: True,
      is_beneficiary_signed: True,
      is_lock_time_passed: True,
    }
  let tx = get_test_tx(test_case)
  vesting.spend(datum, Void, output_reference, tx)
}
```

### Design commentary from the docs

> "The key feature here is the time-based check, which is abstracted by the valid_after function. … transactions can have validity intervals that define from when and until the transaction is considered valid. Validity bounds are checked by the ledger prior to executing a script. … This is meant to give scripts a notion of time, while preserving determinism from within the context of a script. For example, in this scenario, given a lower bound `A` on the transaction, we can deduce that the current time is _at least_ `A`."
>
> — https://aiken-lang.org/example--vesting/mesh

### Alternate worked example: hello_world (no external deps)

For a self-contained primer (no vodka dependency), the hello-world validator is the standard minimal example. Full source from `aiken-lang/site` `src/pages/example--hello-world/basics.mdx`:

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

— Source: https://aiken-lang.org/example--hello-world/basics

---

## 7. Aiken project structure & toolchain

### `aiken new`

```
aiken new foo/bar
cd bar
```

Produces:

```
.
├── README.md
├── aiken.toml
├── env
├── lib
│   └── bar
└── validators
    └── placeholder.ak
```

> "Aiken projects divide their source code in two categories: library code, and application code. Library code must be located in a `lib` folder, and application code (i.e. on-chain validators) located in a `validators` folder."
>
> — https://aiken-lang.org/fundamentals/getting-started

### `aiken.toml` (sample)

```toml
name = "aiken-lang/hello-world"
version = "0.0.0"
compiler = "v1.1.21"
plutus = "v3"
license = "Apache-2.0"
description = "Aiken contracts for project 'aiken-lang/hello-world'"

[repository]
user = 'aiken-lang'
project = 'hello-world'
platform = 'github'

[[dependencies]]
name = "aiken-lang/stdlib"
version = "v3.0.0"
source = "github"

[config]
```

Key fields:
- `name` — `{org}/{repo}` form
- `compiler` — required compiler version (pins toolchain)
- `plutus` — target Plutus version (`v3` is current default)
- `[[dependencies]]` — list of github deps; each has `name`, `version` (tag/branch/sha), `source`
- `[config.default]` — static constants exposed via `use config` (e.g. `operator` byte string)
- `members = ["pkgs/*"]` — workspace/monorepo support (early stage)

— Source: https://aiken-lang.org/fundamentals/getting-started

### CLI workflow

| Command | Effect |
|---|---|
| `aiken new {org}/{repo}` | Scaffold a new project |
| `aiken fmt` | Format source files |
| `aiken check` | Type-check, resolve deps, **run all tests** (preserves traces by default) |
| `aiken build` | Compile to UPLC, emit `plutus.json` (CIP-0057 blueprint). Strips traces by default. |
| `aiken blueprint ...` | Generate addresses, apply parameters, transform blueprint outputs |
| `aiken docs` | Generate HTML docs from type annotations and `///` comments |
| `aiken packages` | Manage dependencies (avoid hand-editing `aiken.toml`) |

Trace control:
- `aiken build --trace-level silent|compact|verbose` — strip, label-only, or full
- `aiken check --trace-level silent` — disable for benchmark parity

> "Use `aiken build` to compile a project, and `aiken check` to only type-check a project and run tests."
>
> — https://aiken-lang.org/fundamentals/getting-started

### Output: `plutus.json` (CIP-0057 blueprint)

> "This generate a CIP-0057 Plutus blueprint as `plutus.json` at the root of your project. This blueprint describes your on-chain contract and its binary interface. In particular, it contains the generated on-chain code that will be executed by the ledger, and a hash of your validator(s) that can be used to construct addresses."
>
> — https://aiken-lang.org/example--hello-world/basics

### Installation

```bash
# Recommended: aikup version manager
curl --proto '=https' --tlsv1.2 -LsSf https://install.aiken-lang.org | sh
aikup install                       # latest
aikup install v1.1.21               # specific version

# or npm
npm install -g @aiken-lang/aikup

# or homebrew
brew install aiken-lang/tap/aikup

# Windows
powershell -c "irm https://windows.aiken-lang.org | iex"
```

— Source: https://aiken-lang.org/installation-instructions

### Well-known packages

- **Prelude** (auto-imported, `aiken-lang/prelude`) — base types & functions.
- **Standard library** (`aiken-lang/stdlib`) — `aiken/*` and `cardano/*` modules (see section 4).
- **Fuzz** (`aiken-lang/fuzz`) — property-based testing / generators / transaction-level state machines.
- **Vodka** (`sidan-lab/vodka`) — helpers like `key_signed`, `valid_after`, and the `mocktail` test framework used in the vesting example.
- Community packages registry: https://packages.aiken-lang.org

— Source: https://aiken-lang.org/fundamentals/getting-started

---

## Appendix A — Additional gotchas worth citing

### Trace-if-false (`?`) operator

```aiken
must_say_hello? && must_be_signed?
```
> "This operator will trace the expression it is attached to only if it evaluates to `False`. This encourages an approach where validators are built as a conjunction or disjunction of requirements. On unsuccessful executions, all the invalidated requirements will leave a trace!"
>
> — https://aiken-lang.org/example--hello-world/basics

### Trace evaluation context

> "Only traces that are actually evaluated by the virtual machine get captured. Expressions short-circuited by logical operators won't produce traces."
>
> — https://aiken-lang.org/language-tour/troubleshooting

### Custom trace messages

> "Doc comments on expect statements generate runtime traces"

```aiken
/// life, universe and everything.
expect answer == 42
```

In `--trace-level compact`, only text before `:` in the label is preserved.

### CBOR diagnostic

> "Diagnostics are meant to be used only in development or for testing; in combination with `trace` for example."

Use `aiken/cbor.diagnostic` for human-readable runtime inspection.

### Hash sizes

> "Hashes are generally 32-byte long on Cardano (or 256 bits), **except for credentials** (i.e. keys or scripts) which are 28-byte long (or 224 bits)."

Maps to: `blake2b_256` for tx ids / data hashes / asset-name receipts; `blake2b_224` for `VerificationKeyHash`, `ScriptHash`, `PolicyId`.

— Source: https://aiken-lang.org/fundamentals/what-i-wish-i-knew

---

## Appendix B — Sources index

| # | URL | Used in |
|---|---|---|
| 1 | https://aiken-lang.org/fundamentals/eutxo | §1, §3, §5.3 |
| 2 | gh:input-output-hk/pogun-claude-meadhall/agents/cardano-mentor/agent.md | §1(e), §5.5, §5.6, §5.7, §5.8 |
| 3 | https://aiken-lang.org/language-tour/primitive-types | §2 |
| 4 | https://aiken-lang.org/language-tour/custom-types | §2, §5.2, §5.7 |
| 5 | https://aiken-lang.org/language-tour/functions | §2 |
| 6 | https://aiken-lang.org/language-tour/modules | §2 |
| 7 | https://aiken-lang.org/language-tour/control-flow | §2 (substitute for /pipelines which 404s) |
| 8 | https://aiken-lang.org/language-tour/validators | §3 |
| 9 | https://aiken-lang.org/example--vesting/mesh | §6 (substitute for /example--vesting which 404s) |
| 10 | https://aiken-lang.org/example--hello-world/basics | §6, Appendix A |
| 11 | https://github.com/aiken-lang/stdlib (lib/) | §4 |
| 12 | https://aiken-lang.org/fundamentals/common-design-patterns | §5.1, §5.6, §5.12 |
| 13 | https://aiken-lang.org/faq | §5 (multiple) |
| 14 | https://aiken-lang.org/language-tour/troubleshooting | §5.4, Appendix A |
| 15 | https://medium.com/@vacuumlabs_auditing/cardano-vulnerabilities-1-double-satisfaction-219f1bc9665e | §5.1 |
| 16 | https://aiken-lang.org/installation-instructions | §7 |
| 17 | https://aiken-lang.org/fundamentals/getting-started | §7 (substitute for /getting-started which 404s) |
| 18 | https://aiken-lang.org/fundamentals/what-i-wish-i-knew | §5.10, §5.11, §5.13, Appendix A |

End of RESEARCH.md.
