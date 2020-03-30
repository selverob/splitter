use rustyline::error::ReadlineError;
use std::process::{Command, Output};

pub fn get_accounts(pattern: &str) -> Result<Vec<String>, ReadlineError> {
    let out = Command::new("ledger")
        .arg("accounts")
        .arg(pattern)
        .output()?;
    process_ledger_output(out)
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
