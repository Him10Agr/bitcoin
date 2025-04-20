use uuid::Uuid;
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::util::MerkleRoot;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transactions {
    pub inputs: Vec<TransactionsInput>,
    pub outputs: Vec<TransactionsOutput>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionsInput {
    pub prev_transaction_output_hash: [u8; 32],
    pub signature: [u8; 64]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionsOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: [u8; 33]
}

impl Transactions {

    pub fn new(
        inputs: Vec<TransactionsInput>,
        outputs: Vec<TransactionsOutput>
    ) -> Self {
        return Transactions { inputs, outputs };
    }

    pub fn hash(self: &Self) -> ! {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: [u8; 32],
    pub merkle_root: MerkleRoot,
    pub target: U256,
}

impl BlockHeader {

    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: [u8; 32],
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {

        return BlockHeader { timestamp, nonce, prev_block_hash, merkle_root, target };
    }

    pub fn hash() -> ! {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transactions>
}

impl Block {

    pub fn new(
        header: BlockHeader,
        transactions: Vec<Transactions>
    ) -> Self {

        return Block {
            header: header,
            transactions: transactions
        };
    }

    pub fn hash(self: &Self) -> ! {
        unimplemented!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockChain {
    pub blocks: Vec<Block>
}

impl BlockChain {

    pub fn new() -> Self {
        return BlockChain{blocks: Vec::new()};
    }

    pub fn add_block(self: &mut Self, block: Block) {
        self.blocks.push(block);
    }
}