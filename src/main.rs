mod ledger;
mod transaction;
mod tui;

fn main() {
    tui::TUIController::new()
        .run()
        .expect("Error when running the TUI");
}
