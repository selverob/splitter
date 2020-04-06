mod ledger;
mod transaction;
mod tui;

use std::env;

fn main() {
    let ledger_filename = env::args().nth(1);
    match ledger_filename {
        Some(filename) => tui::TUIController::new(filename)
            .run()
            .expect("Error when running the TUI"),
        None => println!("Please provide path to your ledger file"),
    };
}
