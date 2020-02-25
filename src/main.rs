mod ledger;
mod transaction;
mod tui;

fn main() {
    println!("{:?}", ledger::get_accounts("by").unwrap());
}
