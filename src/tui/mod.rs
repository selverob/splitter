mod parser;

use std::borrow::Cow::{self, Borrowed, Owned};

use crate::ledger::get_accounts;
use crate::transaction::Transaction;

use rustyline::completion::{extract_word, Completer};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyPress};
use rustyline_derive::{Helper, Validator};

use anyhow::Result;

#[derive(Helper, Validator)]
struct TUIHelper {
    hinter: HistoryHinter,
    highlighter: MatchingBracketHighlighter,
    colored_prompt: String,
}

impl TUIHelper {
    fn new() -> TUIHelper {
        TUIHelper {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "".to_owned(),
        }
    }
}

impl Completer for TUIHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> Result<(usize, Vec<String>), ReadlineError> {
        // The only supported separator is a space, ASCII 32.
        let (word_start, word_to_complete) = extract_word(line, pos, None, &[32u8][..]);
        let words: Vec<&str> = line.split_ascii_whitespace().collect();
        let mut p = parser::Parser::new();
        let mut parsed_characters = 0usize;
        for word in words {
            parsed_characters += word.len() + 1;
            if parsed_characters > word_start {
                break;
            }
            if p.parse_word(word).is_err() {
                return Ok((0, vec![]));
            }
        }
        if p.next != parser::TokenType::Account {
            return Ok((0, vec![]));
        }
        Ok((word_start, get_accounts(word_to_complete)?))
    }
}

impl Hinter for TUIHelper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for TUIHelper {
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

pub struct TUIController {
    current_tx: Option<Transaction>,
    editor: rustyline::Editor<TUIHelper>,
}

impl TUIController {
    pub fn new() -> TUIController {
        let editor_config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .output_stream(OutputStreamType::Stdout)
            .build();
        let mut editor = Editor::with_config(editor_config);
        editor.set_helper(Some(TUIHelper::new()));
        editor.bind_sequence(KeyPress::Meta('N'), Cmd::HistorySearchForward);
        editor.bind_sequence(KeyPress::Meta('P'), Cmd::HistorySearchBackward);
        if editor.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
        TUIController {
            current_tx: None,
            editor,
        }
    }

    pub fn run(&mut self) -> rustyline::Result<()> {
        loop {
            let p = if self.current_tx.is_none() {
                "header> ".to_owned()
            } else {
                "change> ".to_owned()
            };
            self.editor.helper_mut().expect("No helper").colored_prompt =
                format!("\x1b[1;32m{}\x1b[0m", p);
            let line = self.editor.readline(&p);
            match line {
                Ok(line) => {
                    self.editor.add_history_entry(line.clone());
                    let trimmed = line.trim();
                    if self.current_tx.is_none() {
                        self.parse_header(&trimmed);
                    } else {
                        self.parse_change(&trimmed);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    break;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        self.editor.save_history("history.txt")
    }

    fn parse_header(&mut self, line: &str) {
        match parser::parse_transaction_header(line) {
            Ok(transaction) => self.current_tx = Some(transaction),
            Err(err) => println!("{}", err),
        };
    }

    fn parse_change(&mut self, line: &str) {
        if line == "" {
            print!("{}", self.current_tx.as_ref().unwrap());
            self.current_tx = None;
            return;
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
        } else {
            p.operation()
                .unwrap()
                .add_to_transation(&mut self.current_tx.as_mut().unwrap());
        }
    }
}
