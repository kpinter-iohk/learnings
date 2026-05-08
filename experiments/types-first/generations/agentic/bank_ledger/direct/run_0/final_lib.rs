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
    Unbalanced(i128),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::EmptyEntries => write!(f, "transaction has no entries"),
            LedgerError::UnknownAccount(id) => write!(f, "unknown account: {:?}", id),
            LedgerError::Unbalanced(sum) => {
                write!(f, "transaction entries do not sum to zero (sum = {})", sum)
            }
        }
    }
}

struct Account {
    #[allow(dead_code)]
    name: String,
    ty: AccountType,
    balance: i64,
}

struct Transaction {
    #[allow(dead_code)]
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
        self.accounts.insert(
            id,
            Account {
                name,
                ty,
                balance: 0,
            },
        );
        id
    }

    pub fn post(
        &mut self,
        entries: Vec<(AccountId, i64)>,
    ) -> Result<TransactionId, LedgerError> {
        if entries.is_empty() {
            return Err(LedgerError::EmptyEntries);
        }

        for (id, _) in &entries {
            if !self.accounts.contains_key(id) {
                return Err(LedgerError::UnknownAccount(*id));
            }
        }

        let mut sum: i128 = 0;
        for (_, amount) in &entries {
            sum += *amount as i128;
        }
        if sum != 0 {
            return Err(LedgerError::Unbalanced(sum));
        }

        for (id, amount) in &entries {
            if let Some(account) = self.accounts.get_mut(id) {
                account.balance = account.balance.saturating_add(*amount);
            }
        }

        let tx_id = TransactionId(self.next_transaction_id);
        self.next_transaction_id += 1;
        self.transactions.insert(tx_id, Transaction { entries });
        Ok(tx_id)
    }

    pub fn balance(&self, id: AccountId) -> Result<i64, LedgerError> {
        match self.accounts.get(&id) {
            Some(account) => Ok(account.balance),
            None => Err(LedgerError::UnknownAccount(id)),
        }
    }

    pub fn account_type(&self, id: AccountId) -> Option<AccountType> {
        self.accounts.get(&id).map(|a| a.ty)
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}
