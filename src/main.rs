mod ledger;
mod transaction;
mod tui;

fn main() {
    tui::run().expect("Error when running the TUI");
}
