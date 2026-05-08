use solution::{AccountType, Ledger};

/// Returns an AccountId from a separate ledger, well past any ID likely to
/// have been issued in a fresh ledger. Robust to monotonic-from-0
/// implementations: if both ledgers count from 0, this is still id=999.
fn far_unknown_id() -> solution::AccountId {
    let mut other = Ledger::new();
    let mut last = other.open_account("seed_0".into(), AccountType::Asset);
    for i in 1..1000 {
        last = other.open_account(format!("seed_{i}"), AccountType::Asset);
    }
    last
}

#[test]
fn open_account_returns_unique_ids() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Asset);
    assert_ne!(a, b);
}

#[test]
fn balance_starts_at_zero() {
    let mut l = Ledger::new();
    let a = l.open_account("cash".into(), AccountType::Asset);
    assert_eq!(l.balance(a).unwrap(), 0);
}

#[test]
fn account_type_returns_correct_type() {
    let mut l = Ledger::new();
    let a = l.open_account("cash".into(), AccountType::Asset);
    let b = l.open_account("revenue".into(), AccountType::Revenue);
    assert_eq!(l.account_type(a), Some(AccountType::Asset));
    assert_eq!(l.account_type(b), Some(AccountType::Revenue));
}

#[test]
fn balanced_transaction_succeeds() {
    let mut l = Ledger::new();
    let cash = l.open_account("cash".into(), AccountType::Asset);
    let rev = l.open_account("rev".into(), AccountType::Revenue);
    let r = l.post(vec![(cash, 100), (rev, -100)]);
    assert!(r.is_ok());
}

#[test]
fn unbalanced_transaction_errors() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Liability);
    assert!(l.post(vec![(a, 100), (b, -50)]).is_err());
}

#[test]
fn empty_transaction_errors() {
    let mut l = Ledger::new();
    assert!(l.post(vec![]).is_err());
}

#[test]
fn unknown_account_in_transaction_errors() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let bogus = far_unknown_id();
    let r = l.post(vec![(a, 100), (bogus, -100)]);
    assert!(r.is_err());
}

#[test]
fn balance_reflects_posted_entries() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Liability);
    l.post(vec![(a, 100), (b, -100)]).unwrap();
    assert_eq!(l.balance(a).unwrap(), 100);
    assert_eq!(l.balance(b).unwrap(), -100);
}

#[test]
fn balances_accumulate_across_transactions() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Liability);
    l.post(vec![(a, 100), (b, -100)]).unwrap();
    l.post(vec![(a, 50), (b, -50)]).unwrap();
    l.post(vec![(a, -30), (b, 30)]).unwrap();
    assert_eq!(l.balance(a).unwrap(), 120);
    assert_eq!(l.balance(b).unwrap(), -120);
}

#[test]
fn balance_for_unknown_account_errors() {
    let l = Ledger::new();
    let bogus = far_unknown_id();
    assert!(l.balance(bogus).is_err());
}

#[test]
fn account_type_for_unknown_returns_none() {
    let l = Ledger::new();
    let bogus = far_unknown_id();
    assert_eq!(l.account_type(bogus), None);
}

#[test]
fn transaction_with_repeated_account_sums_correctly() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Liability);
    // Two debits to same account, one credit.
    l.post(vec![(a, 100), (a, 50), (b, -150)]).unwrap();
    assert_eq!(l.balance(a).unwrap(), 150);
    assert_eq!(l.balance(b).unwrap(), -150);
}

#[test]
fn three_way_balanced_transaction_succeeds() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Expense);
    let c = l.open_account("c".into(), AccountType::Revenue);
    l.post(vec![(a, 100), (b, 30), (c, -130)]).unwrap();
    assert_eq!(l.balance(a).unwrap(), 100);
    assert_eq!(l.balance(b).unwrap(), 30);
    assert_eq!(l.balance(c).unwrap(), -130);
}

#[test]
fn unbalanced_transaction_does_not_affect_balances() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Liability);
    l.post(vec![(a, 50), (b, -50)]).unwrap();
    assert!(l.post(vec![(a, 999), (b, -1)]).is_err());
    // Balances unchanged from the failed transaction
    assert_eq!(l.balance(a).unwrap(), 50);
    assert_eq!(l.balance(b).unwrap(), -50);
}

#[test]
fn transaction_ids_are_unique() {
    let mut l = Ledger::new();
    let a = l.open_account("a".into(), AccountType::Asset);
    let b = l.open_account("b".into(), AccountType::Liability);
    let t1 = l.post(vec![(a, 1), (b, -1)]).unwrap();
    let t2 = l.post(vec![(a, 1), (b, -1)]).unwrap();
    assert_ne!(t1, t2);
}
