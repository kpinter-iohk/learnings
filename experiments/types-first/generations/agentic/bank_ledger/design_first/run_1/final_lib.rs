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
    Unbalanced(i64),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::EmptyEntries => write!(f, "transaction has no entries"),
            LedgerError::UnknownAccount(id) => {
                write!(f, "unknown account: {:?}", id)
            }
            LedgerError::Unbalanced(sum) => {
                write!(f, "transaction is unbalanced: entries sum to {}", sum)
            }
        }
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
        Ledger {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            next_account_id: 0,
            next_transaction_id: 0,
        }
    }

    pub fn open_account(&mut self, name: String, ty: AccountType) -> AccountId {
        let id = AccountId(self.next_account_id);
        self.next_account_id += 1;
        self.accounts.insert(id, Account { name, ty });
        id
    }

    pub fn post(
        &mut self,
        entries: Vec<(AccountId, i64)>,
    ) -> Result<TransactionId, LedgerError> {
        if entries.is_empty() {
            return Err(LedgerError::EmptyEntries);
        }
        let mut sum: i64 = 0;
        for (acct, amount) in &entries {
            if !self.accounts.contains_key(acct) {
                return Err(LedgerError::UnknownAccount(*acct));
            }
            sum = sum.saturating_add(*amount);
        }
        if sum != 0 {
            return Err(LedgerError::Unbalanced(sum));
        }
        let id = TransactionId(self.next_transaction_id);
        self.next_transaction_id += 1;
        self.transactions.insert(id, Transaction { entries });
        Ok(id)
    }

    pub fn balance(&self, id: AccountId) -> Result<i64, LedgerError> {
        if !self.accounts.contains_key(&id) {
            return Err(LedgerError::UnknownAccount(id));
        }
        let mut total: i64 = 0;
        for tx in self.transactions.values() {
            for (acct, amount) in &tx.entries {
                if *acct == id {
                    total = total.saturating_add(*amount);
                }
            }
        }
        Ok(total)
    }

    pub fn account_type(&self, id: AccountId) -> Option<AccountType> {
        self.accounts.get(&id).map(|a| a.ty)
    }
}
