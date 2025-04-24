use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;
use std::io::{Error as IOError, ErrorKind as IOErrorKind, Read,
    Result as IOResult, Write};

use crate::util::Saveable;



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

impl Saveable for Transactions {
    fn load<I: Read>(reader: I) -> IOResult<Self> {
        ciborium::de::from_reader(reader).map_err(|_| {
            IOError::new(IOErrorKind::InvalidData,
            "Failed to deserialize Transaction")
        })
    }

    fn save<O: Write>(self: &Self, writer: O) -> IOResult<()> {
        ciborium::ser::into_writer(self, writer).map_err(|_| {
            IOError::new(IOErrorKind::InvalidData,
            "Failed to serialize Transaction")
        })
    }
}