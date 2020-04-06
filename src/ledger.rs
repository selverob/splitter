use crate::transaction::Transaction;
use rustyline::error::ReadlineError;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::process::{Command, Output};

pub fn get_accounts(
    path_to_ledger_file: &str,
    pattern: &str,
) -> Result<Vec<String>, ReadlineError> {
    let out = Command::new("ledger")
        .arg("-f")
        .arg(path_to_ledger_file)
        .arg("accounts")
        .arg(pattern)
        .output()?;
    process_ledger_output(out)
}

pub fn write_transaction(path_to_ledger_file: &str, tx: &Transaction) -> io::Result<()> {
    let file = OpenOptions::new().append(true).open(path_to_ledger_file)?;
    write!(&file, "{:}\n", tx)?;
    file.sync_all()
}

fn process_ledger_output(out: Output) -> Result<Vec<String>, ReadlineError> {
    match String::from_utf8(out.stdout) {
        Ok(str_output) => Ok(str_output
            .split('\n')
            .filter(|s| s != &"")
            .map(|s| s.to_owned())
            .collect()),
        Err(_) => Err(ReadlineError::Utf8Error),
    }
}
