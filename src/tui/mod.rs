mod parser;

use std::borrow::Cow::{self, Borrowed, Owned};

use crate::transaction::Transaction;

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyPress};
use rustyline_derive::{Helper, Validator};

use anyhow::anyhow;
use anyhow::Result;
use rust_decimal::Decimal;

#[derive(Helper, Validator)]
struct MyHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

pub fn run() -> rustyline::Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();
    let h = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
    };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(h));
    rl.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let mut tx_out: Option<Transaction> = None;
    loop {
        let tx = &mut tx_out;
        let p = if tx.is_none() {
            "header> ".to_owned()
        } else {
            "change> ".to_owned()
        };
        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);
        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.clone());
                let trimmed = line.trim();
                if tx.is_none() {
                    match parser::parse_transaction_header(trimmed) {
                        Ok(transaction) => tx_out = Some(transaction),
                        Err(err) => println!("{}", err),
                    };
                } else {
                    if trimmed == "" {
                        print!("{}", tx.as_ref().unwrap());
                        tx_out = None;
                        continue;
                    }
                    let mut p = parser::Parser::new();
                    for word in line.split_ascii_whitespace() {
                        let result = p.parse_word(word);
                        if let Err(err) = result {
                            println!("{}", err);
                            continue;
                        }
                    }
                    if p.next != parser::TokenType::EOL {
                        println!("Invalid change command, expecting {:?}", p.next);
                        continue;
                    } else {
                        p.operation()
                            .unwrap()
                            .add_to_transation(&mut tx.as_mut().unwrap());
                    }
                }
                continue;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}
