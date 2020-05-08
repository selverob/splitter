use crate::transaction::Transaction;
use chrono::NaiveDate;
use rustyline::error::ReadlineError;
use std::fs::{rename, File};
use std::io::{Read, Write};
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

pub fn get_commodities(
    path_to_ledger_file: &str,
    starts_with: &str,
) -> Result<Vec<String>, ReadlineError> {
    let out = Command::new("ledger")
        .arg("-f")
        .arg(path_to_ledger_file)
        .arg("commodities")
        .output()?;
    let all_commodities = process_ledger_output(out)?;
    return Ok(all_commodities
        .iter()
        .filter_map(|c| {
            if c.starts_with(starts_with) {
                Some(c.to_owned())
            } else {
                None
            }
        })
        .collect());
}

pub fn write_transaction(path_to_ledger_file: &str, tx: &Transaction) -> Result<(), ReadlineError> {
    let date_ends = get_date_end_positions(path_to_ledger_file)?;
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut file = File::open(path_to_ledger_file)?;
        file.read_to_end(&mut buf)?;
    }

    let tx_pos = get_pos_for_date(date_ends, tx.date);
    let split_offset = if tx_pos < buf.len() - 1 { 1 } else { 0 };

    let (before_tx, after_tx) = buf.split_at(tx_pos + split_offset);
    let tmpfile_path = format!("{}.tmp", path_to_ledger_file);
    let mut tmpfile = File::create(&tmpfile_path)?;
    tmpfile.write_all(&before_tx)?;
    if let Some(last_char) = before_tx.last() {
        if *last_char != 10 || after_tx.len() == 0 {
            writeln!(tmpfile, "")?;
        }
    }

    write!(tmpfile, "{}", tx)?;

    if after_tx.first().is_none() || *after_tx.first().unwrap() != 10 {
        writeln!(tmpfile, "")?;
    }

    tmpfile.write_all(&after_tx)?;
    tmpfile.sync_all()?;
    rename(tmpfile_path, path_to_ledger_file)?;
    Ok(())
}

fn get_pos_for_date(date_ends: Vec<(NaiveDate, usize)>, tx_date: NaiveDate) -> usize {
    match date_ends.binary_search_by_key(&tx_date, |(date, _)| *date) {
        Ok(last_occurrence_index) => date_ends[last_occurrence_index].1,
        Err(larger_occurrence_index) => {
            if larger_occurrence_index == 0 {
                0
            } else {
                date_ends[larger_occurrence_index - 1].1
            }
        }
    }
}

fn get_date_end_positions(
    path_to_ledger_file: &str,
) -> Result<Vec<(NaiveDate, usize)>, ReadlineError> {
    let out = Command::new("ledger")
        .arg("-f")
        .arg(path_to_ledger_file)
        .arg("register")
        .arg("--sort")
        .arg("date,beg_pos")
        .arg("--format")
        .arg("%(date),%(end_pos)\n")
        .output()?;
    let last_positions = process_ledger_output(out)?
        .iter()
        .map(|line| {
            let mut split = line.split(",");
            let date_str = split.next().unwrap();
            let end_pos_str = split.next().unwrap();
            (
                NaiveDate::parse_from_str(date_str, "%Y/%m/%d").unwrap(),
                end_pos_str.parse().unwrap(),
            )
        })
        .fold(
            Vec::new(),
            |mut collected: Vec<(NaiveDate, usize)>, (date, end_pos)| {
                if collected.last().is_some() && collected.last().unwrap().0 == date {
                    collected.pop();
                    collected.push((date, end_pos));
                } else {
                    collected.push((date, end_pos));
                }
                collected
            },
        );
    Ok(last_positions)
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
