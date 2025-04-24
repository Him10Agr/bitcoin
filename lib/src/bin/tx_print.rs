use std::fs::File;
use std::env;
use std::process::exit;

use btclib::types::Transactions;
use btclib::util::Saveable;

fn main() {
    let path = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        eprintln!("Usage: tx_print <tx_file>");
        exit(1);
    };

    if let Ok(file) = File::open(path) {
        let tx = Transactions::load(file)
                            .expect("Failed to load transaction");
        println!("{:?}", tx);                         
    }
}