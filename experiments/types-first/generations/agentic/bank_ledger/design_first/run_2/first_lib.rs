use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(u64);

#[derive(Debug)]
pub enum LedgerError {
    EmptyEntries,
    UnknownAccount(AccountId),
    Unbalanced { sum: i64 },
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

struct Account {
    name: String,
    ty: AccountType,
}

struct Transaction {
    entries: Vec<(AccountId, i64)>,
}

pub struct Ledger {
    accounts: HashMap<AccountId, Account>,
    transactions: HashMap<TransactionId, Transaction>,
    next_account_id: u64,
    next_transaction_id: u64,
}

impl Ledger {
    pub fn new() -> Self {
        todo!()
    }

    pub fn open_account(&mut self, name: String, ty: AccountType) -> AccountId {
        todo!()
    }

    pub fn post(
        &mut self,
        entries: Vec<(AccountId, i64)>,
    ) -> Result<TransactionId, LedgerError> {
        todo!()
    }

    pub fn balance(&self, id: AccountId) -> Result<i64, LedgerError> {
        todo!()
    }

    pub fn account_type(&self, id: AccountId) -> Option<AccountType> {
        todo!()
    }
}
