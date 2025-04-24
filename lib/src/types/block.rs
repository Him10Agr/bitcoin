use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::util::MerkleRoot;
use crate::sha256::Hash;
use crate::error::{BtcError, Result};
use std::collections::HashMap;
use std::io::{Error as IOError, ErrorKind as IOErrorKind, Read,
    Result as IOResult, Write};

use super::{Transactions, TransactionsOutput};
use crate::util::Saveable;

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

impl Saveable for Block {

    fn load<I: Read>(reader: I) -> IOResult<Self> {
        ciborium::de::from_reader(reader).map_err(|_| {
            IOError::new(IOErrorKind::InvalidData,
            "Failed to desrialize Block")
        })
    }

    fn save<O: Write>(self: &Self, writer: O) -> IOResult<()> {
        ciborium::ser::into_writer(self, writer).map_err(|_| {
            IOError::new(IOErrorKind::InvalidData,
            "Failed to serialize Block")
        })
    }
}

