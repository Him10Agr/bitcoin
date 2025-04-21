use crate::U256;
use sha256::digest;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Serialize, Deserialize, 
            Debug, PartialEq, Eq, Hash)]
pub struct Hash(U256);
impl Hash {

    // hash anything that can be serde Serialized via ciborium
    pub fn hash<T: serde::Serialize>(data: &T) -> Self {

        let mut serialized: Vec<u8> = Vec::new();
        if let Err(e) = ciborium::into_writer(data, &mut serialized) {
            panic!(
                "Failed to serialize data: {:?}. \
                 This should not happen", e
            );
        }

        let hash = digest(&serialized);
        let hash_bytes = hex::decode(hash).unwrap();
        let hash_array: [u8; 32] = hash_bytes.as_slice().try_into().unwrap();

        return Hash(U256::from_big_endian(&hash_array));
    }

    //check if a hash matches a target
    pub fn matches_target(self: &Self, target: U256) -> bool {

        return self.0 <= target;
    }

    //zero hash
    pub fn zero() -> Self {
        return Hash(U256::zero());
    }

    pub fn as_bytes(self: &Self) -> [u8; 32] {
        let bytes = self.0.to_little_endian();
        return bytes.as_slice().try_into().unwrap();
    }
}

impl fmt::Display for Hash {

    fn fmt(self: &Self, f: &mut fmt::Formatter) -> fmt::Result {

        return write!(f, "{:x}", self.0);
    }
}