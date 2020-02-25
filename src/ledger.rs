use anyhow::Result;
use std::process::{Command, Output};

pub fn get_accounts(pattern: &str) -> Result<Vec<String>> {
    let out = Command::new("ledger").arg("accounts").arg(pattern).output()?;
    process_ledger_output(out)
}

fn process_ledger_output(out: Output) -> Result<Vec<String>> {
    let str_output = String::from_utf8(out.stdout)?;
    Ok(str_output.split("\n").filter(|s| s != &"").map(|s| s.to_owned()).collect())
}
