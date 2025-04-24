use uuid::Uuid;
use chrono::Utc;
use std::env;
use std::process::exit;

use btclib::types::{Block, BlockHeader, Transactions, TransactionsOutput};
use btclib::util::{Saveable, MerkleRoot};
use btclib::sha256::Hash;
use btclib::crypto::PrivateKey;

fn main() {
    let path = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        eprintln!("Usage: block_gen <block_file>");
        exit(1);
    };

    let priv_key = PrivateKey::new_key();
    let txs = vec![Transactions::new(
        vec![],
        vec![TransactionsOutput {
            unique_id: Uuid::new_v4(),
            value: btclib::INITIAL_REWARD * 10u64.pow(8),
            pubkey: priv_key.public_key(),
        }],
    )];
    let merkle_root = MerkleRoot::calculate(&txs);
    let block = Block::new(
        BlockHeader::new(Utc::now(), 
        0,
        Hash::zero(),
        merkle_root,
        btclib::MIN_TARGET), txs,
    );
    block.save_to_file(path).expect("Failed to save block");
}