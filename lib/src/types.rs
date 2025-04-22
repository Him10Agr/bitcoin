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

    pub fn verify_transaction(self: &Self, 
                            predicted_block_height: u64,
                            utxos: &HashMap<Hash, 
                            TransactionsOutput>) -> Result<()> {
        let mut inputs: HashMap<Hash, TransactionsOutput> = HashMap::new();
        //reject empty blocks
        if self.transactions.is_empty() {
            return Err(BtcError::InvalidTransaction);
        }

        //verify coinbase transaction
        self.verify_coinbase_transaction(predicted_block_height, utxos)?;
        for transaction in self.transactions.iter().skip(1) {
            let mut input_value = 0;
            let mut output_value = 0;
            for input in &transaction.inputs {
                let prev_output = utxos.get(
                    &input.prev_transaction_output_hash,
                );
                if prev_output.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_output.unwrap();
                //preventing same block double spending
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }
                //signature validation
                if !input.signature.verify(&input.prev_transaction_output_hash, &prev_output.pubkey) {
                    return Err(BtcError::InvalidTransaction);
                }

                input_value += prev_output.value;
                inputs.insert(input.prev_transaction_output_hash, prev_output.clone());
            }
            
            for output in &transaction.outputs {
                output_value += output.value;
            }

            //output_value less than input_value is the fee for the miner
            if input_value < output_value {
                return Err(BtcError::InvalidTransaction);
            }
        }

        return Ok(());
    }

    pub fn verify_coinbase_transaction(self: &Self, 
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionsOutput>
    ) -> Result<()> {
        //coinbase tx is the first tx in the block
        let coinbase_transaction = &self.transactions[0];
        if coinbase_transaction.inputs.len() != 0 {
            return Err(BtcError::InvalidTransaction);
        }
        if coinbase_transaction.outputs.len() == 0 {
            return Err(BtcError::InvalidTransaction);
        }

        //yet to implement function to calculate minor fees
        let miner_fees = self.calculate_miner_fees(utxos)?;
        let block_reward = crate::INITIAL_REWARD 
                                * 10u64.pow(8)
                                / 2u64.pow(
                                    (predicted_block_height
                                    / crate::HALVING_INTERVAL) as u32
                                );
        let total_coinbase_outputs: u64 = 
                                coinbase_transaction
                                .outputs
                                .iter()
                                .map(|output| output.value)
                                .sum();
        if total_coinbase_outputs != block_reward + miner_fees {
            return Err(BtcError::InvalidTransaction);
        }

        return Ok(());
    }

    pub fn calculate_miner_fees(
        self: &Self,
        utxos: &HashMap<Hash, TransactionsOutput>
    ) -> Result<u64> {
        let mut inputs: HashMap<Hash, TransactionsOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionsOutput> = HashMap::new();
        //check every transaction after coinbase
        for transaction in self.transactions.iter().skip(1) {
            for input in &transaction.inputs {
                /*inputs do not contain
                the values of the outputs 
                so we need to match inputs
                to outptuts*/
                let prev_output = utxos.get(
                    &input.prev_transaction_output_hash
                );
                if prev_output.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_output.unwrap();
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }
                inputs.insert(
                    input.prev_transaction_output_hash,
                    prev_output.clone()
                );
            }
            for output in &transaction.outputs {
                if outputs.contains_key(&output.hash()) {
                    return Err(BtcError::InvalidTransaction);
                }
                outputs.insert(
                    output.hash(),
                    output.clone(),
                );
            }
        }

        let input_value: u64 = inputs.values()
                                .map(|output| output.value)
                                .sum();
        let output_value: u64 = outputs.values()
                                .map(|output| output.value)
                                .sum();
        
        return Ok(input_value - output_value);
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