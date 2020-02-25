use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct Transaction {
    date: NaiveDate,
    description: String,
    changes: HashMap<String, Decimal>,
}

impl Transaction {
    fn new(date: NaiveDate, description: String) -> Transaction {
        Transaction {
            date,
            description,
            changes: HashMap::new(),
        }
    }

    fn add_change(&mut self, account: String, amount: Decimal) {
        let curr_amount = if self.changes.contains_key(&account) {
            self.changes[&account]
        } else {
            Decimal::new(0, 0)
        };
        self.changes.insert(account, curr_amount + amount);
    }

    fn add_split_change(&mut self, account: String, split_account: String, amount: Decimal) {
        let half = amount / Decimal::new(2, 0);
        self.add_change(account, half);
        self.add_change(split_account, half);
    }

    fn balance(&self) -> Decimal {
        self.changes.values().map(|amount| *amount).sum()
    }

    fn finalize(&mut self, account: String) {
        self.add_change(account, -self.balance());
    }
}

mod test {
    use super::*;
    use rust_decimal_macros::*;

    #[test]
    fn tx_creation() {
        let tx = Transaction::new(NaiveDate::from_ymd(2020, 01, 10), "Test transaction".to_owned());
        assert_eq!(tx.date, NaiveDate::from_ymd(2020, 01, 10));
        assert_eq!(tx.description, "Test transaction".to_owned());
        assert_eq!(tx.changes.len(), 0);
    }

    #[test]
    fn simple_changes() {
        let mut tx = Transaction::new(NaiveDate::from_ymd(2020, 01, 10), "Test transaction".to_owned());
        tx.add_change("Expenses::Food".to_owned(), dec!(5.95));
        tx.add_change("Expenses::Hygiene".to_owned(), dec!(3.90));
        tx.add_change("Expenses::Food".to_owned(), dec!(2));
        assert_eq!(tx.changes["Expenses::Food"], dec!(7.95));
        assert_eq!(tx.changes["Expenses::Hygiene"], dec!(3.90));
        assert_eq!(tx.balance(), dec!(11.85))
    }

    #[test]
    fn split_changes() {
        let mut tx = Transaction::new(NaiveDate::from_ymd(2020, 01, 10), "Test transaction".to_owned());
        tx.add_split_change("Expenses::Food".to_owned(), "Debts::Peter".to_owned(), dec!(7));
        tx.add_change("Expenses::Food".to_owned(), dec!(2));
        assert_eq!(tx.changes["Expenses::Food"], dec!(5.50));
        assert_eq!(tx.changes["Debts::Peter"], dec!(3.50));
        assert_eq!(tx.balance(), dec!(9))
    }

    #[test]
    fn finalization() {
        let mut tx = Transaction::new(NaiveDate::from_ymd(2020, 01, 10), "Test transaction".to_owned());
        tx.add_change("Expenses::Food".to_owned(), dec!(7));
        tx.add_change("Assets::Cash".to_owned(), dec!(-2));
        tx.finalize("Assets::Account".to_owned());
        assert_eq!(tx.changes["Assets::Account"], dec!(-5));
        assert_eq!(tx.balance(), dec!(0));
    }
}
