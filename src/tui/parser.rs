use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct Amount<'a>(&'a str, Decimal);

#[derive(Clone, Debug, PartialEq)]
pub enum Operation<'a> {
    AddSimpleChange(&'a str, Amount<'a>),
    AddSplitChange(&'a str, &'a str, Amount<'a>),
    Finalize(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenType {
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
    next: TokenType,
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
                Amount(self.currency.unwrap(), self.amount.unwrap()),
            ),
            OperationType::AddSplit => Operation::AddSplitChange(
                self.accounts[0],
                self.accounts[1],
                Amount(self.currency.unwrap(), self.amount.unwrap()),
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
            static ref ACC_RE: Regex = Regex::new("\\p{L}([\\p{L}[:digit:]:])*").unwrap();
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
            static ref CURR_RE: Regex = Regex::new("[^0-9]+").unwrap();
        }
        if CURR_RE.is_match(word) {
            self.currency = Some(word);
        } else {
            return Err(anyhow!("Curreny contains invalid characters"));
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
