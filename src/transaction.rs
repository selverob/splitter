use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub String, pub Decimal);

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub date: NaiveDate,
    pub description: String,
    pub changes: HashMap<String, Vec<Amount>>,
}

impl Transaction {
    pub fn new(date: NaiveDate, description: String) -> Transaction {
        Transaction {
            date,
            description,
            changes: HashMap::new(),
        }
    }

    pub fn add_change(&mut self, account: &str, amount: Amount) {
        self.changes
            .entry(account.to_owned())
            .and_modify(|v| {
                if let Some(pos) = v.iter().position(|am| am.0 == amount.0) {
                    v[pos].1 = v[pos].1 + amount.1;
                } else {
                    v.push(amount.clone());
                    v.sort();
                }
            })
            .or_insert(vec![amount]);
    }

    pub fn add_split_change(&mut self, account: &str, split_account: &str, amount: Amount) {
        let half = Amount(amount.0, amount.1 / dec!(2));
        self.add_change(account, half.clone());
        self.add_change(split_account, half);
    }

    pub fn balance(&self) -> Vec<Amount> {
        let mut balances = HashMap::new();
        for amounts in self.changes.values() {
            for amount in amounts {
                balances
                    .entry(&amount.0)
                    .and_modify(|a: &mut Decimal| *a = amount.1 + *a)
                    .or_insert(amount.1);
            }
        }
        let mut balance_vec: Vec<Amount> = balances
            .iter()
            .map(|(currency, balance)| Amount(currency.to_string(), *balance))
            .collect();
        balance_vec.sort();
        balance_vec
    }

    pub fn finalize(&mut self, account: &str) {
        for amount in self.balance() {
            self.add_change(account, Amount(amount.0, -amount.1));
        }
    }

    fn amounts(&self) -> Vec<(&str, &Amount)> {
        let mut amount_vec = Vec::new();
        for (account, amounts) in &self.changes {
            for amount in amounts {
                amount_vec.push((account.as_ref(), amount));
            }
        }
        amount_vec
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {}",
            self.date.format("%Y-%m-%d").to_string(),
            self.description
        )?;
        let (mut credits, mut debits): (Vec<(&str, &Amount)>, Vec<(&str, &Amount)>) = self
            .amounts()
            .iter()
            .partition(|amount_triple| (amount_triple.1).1 >= dec!(0));
        credits.sort_by_key(|amount_triple| amount_triple.0);
        debits.sort_by_key(|amount_triple| amount_triple.0);
        for (account, amount) in credits {
            writeln!(f, "\t{}\t{} {}", account, amount.0, amount.1)?;
        }
        for (account, amount) in debits {
            writeln!(f, "\t{}\t{} {}", account, amount.0, amount.1)?;
        }
        Ok(())
    }
}

mod test {
    use super::*;

    #[test]
    fn tx_creation() {
        let tx = Transaction::new(
            NaiveDate::from_ymd(2020, 01, 10),
            "Test transaction".to_owned(),
        );
        assert_eq!(tx.date, NaiveDate::from_ymd(2020, 01, 10));
        assert_eq!(tx.description, "Test transaction".to_owned());
        assert_eq!(tx.changes.len(), 0);
    }

    #[test]
    fn simple_changes() {
        let mut tx = Transaction::new(
            NaiveDate::from_ymd(2020, 01, 10),
            "Test transaction".to_owned(),
        );
        tx.add_change(&"Expenses::Food", Amount("€".to_owned(), dec!(5.95)));
        tx.add_change(&"Expenses::Hygiene", Amount("€".to_owned(), dec!(3.90)));
        tx.add_change(&"Expenses::Hygiene", Amount("CZK".to_owned(), dec!(25)));
        tx.add_change(&"Expenses::Hygiene", Amount("CZK".to_owned(), dec!(13)));
        tx.add_change(&"Expenses::Food", Amount("€".to_owned(), dec!(2)));
        tx.add_change(&"Expenses::Food", Amount("CZK".to_owned(), dec!(120)));
        assert_eq!(
            tx.changes["Expenses::Food"],
            vec![
                Amount("CZK".to_owned(), dec!(120)),
                Amount("€".to_owned(), dec!(7.95))
            ]
        );
        assert_eq!(
            tx.changes["Expenses::Hygiene"],
            vec![
                Amount("CZK".to_owned(), dec!(38)),
                Amount("€".to_owned(), dec!(3.90))
            ]
        );
        assert_eq!(
            tx.balance(),
            vec![
                Amount("CZK".to_owned(), dec!(158)),
                Amount("€".to_owned(), dec!(11.85))
            ]
        );
    }

    #[test]
    fn split_changes() {
        let mut tx = Transaction::new(
            NaiveDate::from_ymd(2020, 01, 10),
            "Test transaction".to_owned(),
        );
        tx.add_split_change(
            "Expenses::Food",
            "Debts::Peter",
            Amount("€".to_owned(), dec!(7)),
        );
        tx.add_split_change(
            "Expenses::Food",
            "Debts::Peter",
            Amount("CZK".to_owned(), dec!(120)),
        );
        tx.add_change("Expenses::Food", Amount("€".to_owned(), dec!(2)));
        assert_eq!(
            tx.changes["Expenses::Food"],
            vec![
                Amount("CZK".to_owned(), dec!(60)),
                Amount("€".to_owned(), dec!(5.50))
            ]
        );
        assert_eq!(
            tx.changes["Debts::Peter"],
            vec![
                Amount("CZK".to_owned(), dec!(60)),
                Amount("€".to_owned(), dec!(3.50))
            ]
        );
        assert_eq!(
            tx.balance(),
            vec![
                Amount("CZK".to_owned(), dec!(120)),
                Amount("€".to_owned(), dec!(9))
            ]
        )
    }

    #[test]
    fn finalization() {
        let mut tx = Transaction::new(
            NaiveDate::from_ymd(2020, 01, 10),
            "Test transaction".to_owned(),
        );
        tx.add_change("Expenses::Food", Amount("€".to_owned(), dec!(7)));
        tx.add_change("Expenses::Food", Amount("CZK".to_owned(), dec!(500)));
        tx.add_change("Assets::Cash", Amount("€".to_owned(), dec!(-2)));
        tx.add_change("Assets::Cash", Amount("CZK".to_owned(), dec!(-400)));
        tx.finalize("Assets::Account");
        assert_eq!(
            tx.changes["Assets::Account"],
            vec![
                Amount("CZK".to_owned(), dec!(-100)),
                Amount("€".to_owned(), dec!(-5))
            ]
        );
        assert_eq!(
            tx.balance(),
            vec![
                Amount("CZK".to_owned(), dec!(0)),
                Amount("€".to_owned(), dec!(0))
            ]
        );
    }
}
