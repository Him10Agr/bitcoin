use uuid::Uuid;
use std::env;
use std::process::exit;

use btclib::types::{Transactions, TransactionsOutput};
use btclib::util::Saveable;
use btclib::crypto::PrivateKey;

fn main() {
    let path = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        eprintln!("Usage: tx_gen <tx_file>");
        exit(1);
    };

    let priv_key = PrivateKey::new_key();
    let txs = Transactions::new(
        vec![],
        vec![TransactionsOutput {
            unique_id: Uuid::new_v4(),
            value: btclib::INITIAL_REWARD * 10u64.pow(8),
            pubkey: priv_key.public_key(),
        }],
    );
    txs.save_to_file(path).expect("Failed to save transaction");
}