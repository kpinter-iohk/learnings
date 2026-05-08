Build a double-entry ledger in Rust.

The ledger holds accounts and transactions. Each account has a name and a type. Each transaction is a list of entries; each entry references an account and an amount (positive or negative `i64`). The sum of all entries in a transaction must equal zero.

The `AccountType` enum must have exactly these variants (the variant names are part of the public API):

```rust
pub enum AccountType { Asset, Liability, Equity, Revenue, Expense }
```

`AccountType` must derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`.

Required public API:

- `Ledger::new() -> Self`
- `ledger.open_account(&mut self, name: String, ty: AccountType) -> AccountId`
- `ledger.post(&mut self, entries: Vec<(AccountId, i64)>) -> Result<TransactionId, LedgerError>`
- `ledger.balance(&self, id: AccountId) -> Result<i64, LedgerError>` — sum of all amounts posted to that account
- `ledger.account_type(&self, id: AccountId) -> Option<AccountType>`

`AccountId` and `TransactionId` are public types whose internal shape is your choice. Both must implement `Copy`, `Clone`, `Eq`, `PartialEq`, `Hash`, `Debug`.

`LedgerError` is a public type whose shape is your choice. It must implement `Debug` and `Display`. It must distinguish at minimum: an unbalanced transaction (entries don't sum to zero), an unknown account reference, and an empty entry list.

Behaviors `post` must enforce:
- Empty `entries` → error.
- Any entry referencing an unknown `AccountId` → error.
- Sum of amounts not equal to zero → error.
- All accounts known and entries balance → success; transaction is recorded; return a fresh `TransactionId`.

`balance` returns `Err` for an unknown `AccountId`, otherwise returns the sum of amounts posted to that account across all transactions.

`account_type` returns the type of a known account or `None` for an unknown ID.

Standard library only. The complete program is one `lib.rs`.
