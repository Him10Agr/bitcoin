use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;


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
