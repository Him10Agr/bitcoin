use uuid::Uuid;
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::util::MerkleRoot;
use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;
use crate::error::{BtcError, Result};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionsInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionsOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: PublicKey
}

impl TransactionsOutput {
    
    pub fn hash(self: &Self) -> Hash {
        return Hash::hash(self);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transactions {
    pub inputs: Vec<TransactionsInput>,
    pub outputs: Vec<TransactionsOutput>
}

impl Transactions {

    pub fn new(
        inputs: Vec<TransactionsInput>,
        outputs: Vec<TransactionsOutput>
    ) -> Self {
        return Transactions { inputs, outputs };
    }

    pub fn hash(self: &Self) -> Hash {
        return Hash::hash(self);
    }
}



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: Hash,
    pub merkle_root: MerkleRoot,
    pub target: U256,
}

impl BlockHeader {

    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {

        return BlockHeader { timestamp, nonce, prev_block_hash, merkle_root, target };
    }

    pub fn hash(self: &Self) -> Hash {
        return Hash::hash(self);
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

    pub fn hash(self: &Self) -> Hash {
        return Hash::hash(self);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockChain {
    pub utxos: HashMap<Hash, TransactionsOutput>,
    pub blocks: Vec<Block>
}

impl BlockChain {

    pub fn new() -> Self {
        return BlockChain{utxos: HashMap::new(), blocks: Vec::new()};
    }

    pub fn add_block(self: &mut Self, block: Block) -> Result<()> {

        if self.blocks.is_empty() {
            //if this is first block, check if block's
            //prev_block_hash is all zeros
            if block.header.prev_block_hash != Hash::zero() {
                println!("zero hash");
                return Err(BtcError::InvalidBlock);
            }
        } else {
            //if this is not first block, check if block's 
            //prev_block_hash is hash of the last block
            let last_block = self.blocks.last().unwrap();
            if block.header.prev_block_hash != last_block.hash() {
                println!("previous hash is wrong");
                return Err(BtcError::InvalidBlock);
            }

            //block's hash is less than the target
            if !block.header.hash().matches_target(block.header.target) {
                println!("doesn't match target");
                return Err(BtcError::InvalidBlock);
            }

            //block's merkle root is correct
            let calculated_merkle_root = MerkleRoot::calculate(&block.transactions);
            if calculated_merkle_root != block.header.merkle_root {
                println!("invalid merkle root");
                return Err(BtcError::InvalidMerkleRoot);
            }

            //block's timestamp is after the last block timestamp
            if block.header.timestamp <= last_block.header.timestamp
            {
                return Err(BtcError::InvalidBlock);
            }

            //verify all transactions in the block
            todo!();
        }
        self.blocks.push(block);
        return Ok(());
    }

    pub fn rebuild_utxos(self: &mut Self) {

        for block in &self.blocks {
            for transaction in &block.transactions {
                for input in &transaction.inputs {
                    self.utxos.remove(
                        &input.prev_transaction_output_hash,
                    );
                }
                for output in transaction.outputs.iter() {
                    self.utxos.insert(transaction.hash(), output.clone());
                }
            }
        }
    }
}