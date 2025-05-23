/*
use uuid::Uuid;
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::util::MerkleRoot;
use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;
use crate::error::{BtcError, Result};
use std::collections::{HashMap, HashSet};
use bigdecimal::BigDecimal;

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

    pub fn mine(self: &mut Self, steps: usize) -> bool {
        //if block already matches target
        if self.hash().matches_target(self.target) {
            return true;
        }
        for _ in 0..steps {
            if let Some(new_nonce) = self.nonce.checked_add(1) {
                self.nonce = new_nonce;
            } else {
                self.nonce = 0;
                self.timestamp = Utc::now();
            }

            if self.hash().matches_target(self.target) {
                return true;
            }
        }
        return false;
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
                            (bool, TransactionsOutput)>) -> Result<()> {
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
                ).map(|(_, output)| output);
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
        utxos: &HashMap<Hash, (bool, TransactionsOutput)>
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
        utxos: &HashMap<Hash, (bool, TransactionsOutput)>
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
                ).map(|(_, output)| output);
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
    utxos: HashMap<Hash, (bool, TransactionsOutput)>,
    target: U256,
    blocks: Vec<Block>,
    #[serde(default, skip_serializing)]
    mempool: Vec<(DateTime<Utc>, Transactions)>
}

impl BlockChain {

    pub fn new() -> Self {
        return BlockChain{utxos: HashMap::new(),
                          target: crate::MIN_TARGET,
                          blocks: Vec::new(),
                          mempool: Vec::new()};
    }

    pub fn utxos(self: &Self) -> &HashMap<Hash, (bool, TransactionsOutput)> {
        return &self.utxos;
    }

    pub fn target(self: &Self) -> U256 {
        return self.target;
    }

    pub fn blocks(self: &Self) -> impl Iterator<Item = &Block> {
        return self.blocks.iter();
    }

    pub fn block_height(self: &Self) -> u64 {
        return self.blocks.len() as u64;
    }

    pub fn mempool(self: &Self) -> &[(DateTime<Utc>, Transactions)] {
        //need to implement time tracking
        return &self.mempool;
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
            block.verify_transaction(self.block_height()
                            , &self.utxos)?;
        }

        //Remove tx from mempool that are now in the block
        let block_transactions: HashSet<_> = block
                                    .transactions
                                    .iter()
                                    .map(|tx| tx.hash())
                                    .collect();
        self.mempool.retain(|(_, tx)| {
            !block_transactions.contains(&tx.hash())
        });
        self.blocks.push(block);
        self.try_adjust_target();
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
                    self.utxos.insert(output.hash(), (false, output.clone()));
                }
            }
        }
    }

    //try to adjust the target of the blockchain
    pub fn try_adjust_target(self: &mut Self) {
        if self.blocks.is_empty() {
            return;
        }
        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INETRVAL as usize != 0 {
            return;
        }

        //time to mine the last crate::DIFFICULTY_UPDATE_INTERVAL blocks
        let start_time = self.blocks[self.blocks.len()
            - crate::DIFFICULTY_UPDATE_INETRVAL as usize]
            .header
            .timestamp;
        let end_time = self.blocks.last().unwrap().header.timestamp;
        let time_diff = (end_time - start_time).num_seconds();
        let target_seconds = crate::IDEAL_BLOCK_TIME
                        * crate::DIFFICULTY_UPDATE_INETRVAL;
        //multiply the current target by actual time divided by
        //ideal time

        /*let new_target = self.target
            * (time_diff as f64 / target_seconds as f64) as usize; */
        
        let new_target = BigDecimal::parse_bytes(
            &self.target.to_string().as_bytes(), 10)
            .expect("BUG: impossible")
            *(BigDecimal::from(time_diff)
               / BigDecimal::from(target_seconds));
        
        let new_target_str = new_target.to_string()
                                        .split(".")
                                        .next()
                                        .expect("BUG: Expected a decimal point")
                                        .to_owned();
        
        let new_target = U256::from_str_radix(&new_target_str, 10)
                                                            .expect("BUG: Impossible");

        //4 * self.target > new_target > self.target / 4
        let new_target = if new_target < self.target / 4 {
            self.target / 4
        }else if new_target > self.target * 4 {
            self.target * 4
        } else {
            new_target
        };

        //if new_target > minimum target
        //set it to the minmum target
        self.target = new_target.min(crate::MIN_TARGET);

    }

    pub fn add_to_mempool(self: &mut Self, transaction: Transactions) -> Result<()> {

        //validation pf tx before insertion
        //all inputs must match known UTXO's and must be unique
        let mut known_inputs = HashSet::new();
        for input in &transaction.inputs {
            if !self.utxos.contains_key(
                &input.prev_transaction_output_hash,
            ) {
                println!("Utxos not found");
                dbg!(&self.utxos());
                return Err(BtcError::InvalidTransaction);
            }
            if known_inputs.contains(
                &input.prev_transaction_output_hash
            ) {
                println!("Duplicate input");
                return Err(BtcError::InvalidTransaction);
            }
            known_inputs.insert(input.prev_transaction_output_hash);
        }

        /*if any utxos have bool set to true find the tx that ref them in mempool
        , remove it, and set all the utxos it references to false*/
        for input in &transaction.inputs {
            if let Some((true, _)) = self.utxos.get(&input.prev_transaction_output_hash) {
                /*tx that ref the utxos we are trying to ref*/
                let ref_tx = self.mempool.iter().enumerate()
                                                                .find(|(_, (_, tx))| {
                                                                    tx.outputs.iter().any(|output| {
                                                                        output.hash() == input.prev_transaction_output_hash
                                                                    })
                                                                });
                
                //unmark utxos if found
                if let Some((idx,( _, ref_tx))) = ref_tx {
                    for input in &ref_tx.inputs {
                        //setting all utxos from this tx to false
                        self.utxos.entry(input.prev_transaction_output_hash)
                        .and_modify(|(marked, _)| {
                            *marked = false;
                        });
                    }
                    //remove the tx from mempool
                    self.mempool.remove(idx);
                } else {
                    //if somehow there is no matching tx
                    //set this utxo to false
                    self.utxos.entry(input.prev_transaction_output_hash)
                    .and_modify(|(marked, _)| {
                        *marked = false;
                    });
                }
            }
        }

        //all inputs must be lower than all outputs
        let all_inputs = transaction.inputs.iter()
                                .map(|input| {
                                        self.utxos.get(&input.prev_transaction_output_hash)
                                        .expect("BUG: Impossible")
                                        .1.value
                                }).sum::<u64>();

        let all_outputs = transaction.outputs.iter()
                                    .map(|output| output.value)
                                    .sum();
        
        if all_inputs < all_outputs {
            println!("Inputs are lower than outputs");
            return Err(BtcError::InvalidTransaction);
        }

        //Marking Utxos as used
        for input in &transaction.inputs {
            self.utxos.entry(input.prev_transaction_output_hash)
                    .and_modify(|(marked, _)| {
                        *marked = true;
                    });
        }

        //push tx to mempool
        self.mempool.push((Utc::now(), transaction));
        //sort by miner fee
        self.mempool.sort_by_key(|(_, transaction)| {
            let all_inputs = transaction.inputs.iter()
                                    .map(|input| {
                                        self.utxos.get(&input.prev_transaction_output_hash)
                                        .expect("BUG: Impossible")
                                        .1.value
                                    }).sum::<u64>();
            
            let all_outputs: u64 = transaction.outputs.iter()
                                    .map(|output| output.value)
                                    .sum();
            let miner_fee = all_inputs - all_outputs;
            return miner_fee;
        });
        return Ok(());                 
    }

    //remove tx older than MAX_MEMPOOL_TX_AGE
    pub fn cleanup_mempool(self: &mut Self) {
        let now = Utc::now();
        let mut utxos_hashes_to_unmark: Vec<Hash> = Vec::new();
        self.mempool.retain(|(timestamp, tx)| {
            if now - *timestamp > chrono::Duration::seconds(crate::MAX_MEMPOOL_TRANSACTION_AGE as i64) {
                //push all utxos to unmark to the vector to be unmarked later
                utxos_hashes_to_unmark.extend(tx.inputs.iter().map(
                    |input| {
                        input.prev_transaction_output_hash
                    },
                ));
                false
            } else {
                true
            }
        });

        //unmark all the utxos
        for hash in utxos_hashes_to_unmark {
            self.utxos.entry(hash).and_modify(|(marked, _)| {
                *marked = false;
            },);
        }
    }
}
*/

mod block;
mod blockchain;
mod transaction;

pub use block::{Block, BlockHeader};
pub use blockchain::BlockChain;
pub use transaction::{Transactions, TransactionsInput, TransactionsOutput};