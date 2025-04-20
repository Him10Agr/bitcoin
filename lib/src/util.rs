use crate::sha256::Hash;
use crate::types::Transactions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerkleRoot(Hash);
impl MerkleRoot {

    //calculate the merkle root of a block's transactions
    pub fn calculate(
        transactions: &[Transactions],
    ) -> Self {
        let mut layer: Vec<Hash> = Vec::new();
        for transaction in transactions {
            layer.push(Hash::hash(transaction));
        }

        while layer.len() > 1 {
            let mut new_layer = Vec::new();
            for pair in layer.chunks(2) {
                let left = pair[0];
                let right = pair.get(1).unwrap_or(&pair[0]);
                new_layer.push(Hash::hash(&[left, *right]));
            }
            layer = new_layer;
        }
        return MerkleRoot(layer[0])
    }
}