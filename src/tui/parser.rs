use crate::transaction::{Amount, Transaction};
use anyhow::anyhow;
use anyhow::Result;
use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation<'a> {
    AddSimpleChange(&'a str, Amount),
    AddSplitChange(&'a str, &'a str, Amount),
    Finalize(&'a str),
}

impl<'a> Operation<'a> {
    pub fn add_to_transation(self, tx: &mut Transaction) {
        match self {
            Operation::AddSimpleChange(account, amount) => tx.add_change(account, amount),
            Operation::AddSplitChange(account1, account2, amount) => {
                tx.add_split_change(account1, account2, amount)
            }
            Operation::Finalize(account) => tx.finalize(account),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    Operation,
    Account,
    Currency,
    Amount,
    EOL,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperationType {
    AddSimple,
    AddSplit,
    Finalize,
}

impl OperationType {
    fn parse(word: &str) -> Result<OperationType> {
        match word {
            "a" => Ok(OperationType::AddSimple),
            "s" => Ok(OperationType::AddSplit),
            "f" => Ok(OperationType::Finalize),
            _ => Err(anyhow!("Invalid operation type")),
        }
    }
}

pub struct Parser<'a> {
    pub next: TokenType,
    op_type: Option<OperationType>,
    accounts: Vec<&'a str>,
    currency: Option<&'a str>,
    amount: Option<Decimal>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Parser<'a> {
        Parser {
            next: TokenType::Operation,
            op_type: None,
            accounts: Vec::new(),
            currency: None,
            amount: None,
        }
    }

    pub fn operation(self) -> Option<Operation<'a>> {
        if self.next != TokenType::EOL {
            return None;
        }
        let op = match self.op_type.unwrap() {
            OperationType::AddSimple => Operation::AddSimpleChange(
                self.accounts[0],
                Amount(self.currency.unwrap().to_owned(), self.amount.unwrap()),
            ),
            OperationType::AddSplit => Operation::AddSplitChange(
                self.accounts[0],
                self.accounts[1],
                Amount(self.currency.unwrap().to_owned(), self.amount.unwrap()),
            ),
            OperationType::Finalize => Operation::Finalize(self.accounts[0]),
        };
        Some(op)
    }

    pub fn parse_word(&mut self, word: &'a str) -> Result<()> {
        match self.next {
            TokenType::Operation => self.parse_op_type(word)?,
            TokenType::Account => self.parse_account(word)?,
            TokenType::Currency => self.parse_currency(word)?,
            TokenType::Amount => self.parse_amount(word)?,
            TokenType::EOL => return Err(anyhow!("Unexpected input at end of line")),
        }
        Ok(())
    }

    fn parse_op_type(&mut self, word: &'a str) -> Result<()> {
        self.op_type = Some(OperationType::parse(word)?);
        self.next = TokenType::Account;
        Ok(())
    }

    fn parse_account(&mut self, word: &'a str) -> Result<()> {
        lazy_static! {
            static ref ACC_RE: Regex =
                Regex::new("^[\\p{L}&&[^:digit:]][\\p{L}[:digit:]:]*$").unwrap();
        }
        if ACC_RE.is_match(word) {
            self.accounts.push(word);
        } else {
            return Err(anyhow!("Account name contains invalid character"));
        }
        if self.op_type == Some(OperationType::AddSplit) && self.accounts.len() == 1 {
            self.next = TokenType::Account;
        } else if self.op_type == Some(OperationType::Finalize) {
            self.next = TokenType::EOL;
        } else {
            self.next = TokenType::Currency;
        }
        Ok(())
    }

    fn parse_currency(&mut self, word: &'a str) -> Result<()> {
        lazy_static! {
            static ref CURR_RE: Regex = Regex::new("^[^0-9]+$").unwrap();
        }
        if CURR_RE.is_match(word) {
            self.currency = Some(word);
        } else {
            return Err(anyhow!("Currency contains invalid characters"));
        }
        self.next = TokenType::Amount;
        Ok(())
    }

    fn parse_amount(&mut self, word: &'a str) -> Result<()> {
        self.amount = Some(Decimal::from_str(word)?);
        self.next = TokenType::EOL;
        Ok(())
    }
}

pub fn parse_transaction_header(line: &str) -> Result<Transaction> {
    let fields: Vec<&str> = line.split_ascii_whitespace().collect();
    if fields.is_empty() {
        return Err(anyhow!("No transaction header provided"));
    }
    let date: NaiveDate = fields[0].parse()?;
    let description = fields[1..fields.len()].join(" ");
    Ok(Transaction::new(date, description))
}

mod test {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use rust_decimal_macros::*;

    #[test]
    fn parse_simple() {
        let line = vec!["a", "Expenses", "€", "12.34"];
        let mut parser = Parser::new();
        assert_eq!(parser.next, TokenType::Operation);
        assert!(parser.parse_word(line[0]).is_ok());
        assert_eq!(parser.next, TokenType::Account);
        assert!(parser.parse_word(line[1]).is_ok());
        assert_eq!(parser.next, TokenType::Currency);
        assert!(parser.parse_word(line[2]).is_ok());
        assert_eq!(parser.next, TokenType::Amount);
        assert!(parser.parse_word(line[3]).is_ok());
        assert_eq!(parser.next, TokenType::EOL);
        assert!(parser.parse_word("blah").is_err());
        assert_eq!(
            parser.operation().unwrap(),
            Operation::AddSimpleChange("Expenses", Amount("€".to_owned(), dec!(12.34)))
        );
    }

    #[test]
    fn parse_split() {
        let line = vec!["s", "Expenses", "Debts:Peter", "CZK", "120.50"];
        let mut parser = Parser::new();
        assert_eq!(parser.next, TokenType::Operation);
        assert!(parser.parse_word(line[0]).is_ok());
        assert_eq!(parser.next, TokenType::Account);
        assert!(parser.parse_word(line[1]).is_ok());
        assert_eq!(parser.next, TokenType::Account);
        assert!(parser.parse_word(line[2]).is_ok());
        assert_eq!(parser.next, TokenType::Currency);
        assert!(parser.parse_word(line[3]).is_ok());
        assert_eq!(parser.next, TokenType::Amount);
        assert!(parser.parse_word(line[4]).is_ok());
        assert_eq!(parser.next, TokenType::EOL);
        assert!(parser.parse_word("blah").is_err());
        assert_eq!(
            parser.operation().unwrap(),
            Operation::AddSplitChange(
                "Expenses",
                "Debts:Peter",
                Amount("CZK".to_owned(), dec!(120.50))
            )
        );
    }

    #[test]
    fn parse_finalize() {
        let line = vec!["f", "Accounts:Checking"];
        let mut parser = Parser::new();
        assert_eq!(parser.next, TokenType::Operation);
        assert!(parser.parse_word(line[0]).is_ok());
        assert_eq!(parser.next, TokenType::Account);
        assert!(parser.parse_word(line[1]).is_ok());
        assert_eq!(parser.next, TokenType::EOL);
        assert!(parser.parse_word("blah").is_err());
        assert_eq!(
            parser.operation().unwrap(),
            Operation::Finalize("Accounts:Checking")
        );
    }

    #[test]
    fn test_errors() {
        let mut parser = Parser::new();
        assert!(parser.parse_word("x").is_err());
        assert!(parser.parse_word("a").is_ok());
        assert!(parser.parse_word("123").is_err());
        assert!(parser.parse_word("1checking").is_err());
        assert!(parser.parse_word("Expenses").is_ok());
        assert!(parser.parse_word("123").is_err());
        assert!(parser.parse_word("€").is_ok());
        assert!(parser.parse_word("1a2b").is_err());
        assert!(parser.parse_word("12.30").is_ok());
        assert_eq!(
            parser.operation().unwrap(),
            Operation::AddSimpleChange("Expenses", Amount("€".to_owned(), dec!(12.30)))
        );
    }

    #[test]
    fn tx_header() {
        let tx = parse_transaction_header("2020-02-27 Test transaction").unwrap();
        assert_eq!(tx.date, NaiveDate::from_ymd(2020, 2, 27));
        assert_eq!(tx.description, "Test transaction");
    }
}
