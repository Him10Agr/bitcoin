// network.rs
use serde::{Deserialize, Serialize};
use std::io::{Error as IOError, Read, Write};

use crate::crypto::PublicKey;
use crate::types::{Block, Transactions, TransactionsOutput};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    /// Fetch all UTXOs belonging to a public key
    FetchUTXOs(PublicKey),
    /// UTXOs belonging to a public key. Bool determines if marked
    UTXOs(Vec<(TransactionsOutput, bool)>),
    /// Send a Transactions to the network
    SubmitTransaction(Transactions),
    /// Broadcast a new Transactions to other nodes
    NewTransaction(Transactions),
    /// Ask the node to prepare the optimal block template
    /// with the coinbase Transactions paying the specified
    /// public key
    FetchTemplate(PublicKey),
    /// The template
    Template(Block),
    /// Ask the node to validate a block template.
    /// This is to prevent the node from mining an invalid
    /// block (e.g. if one has been found in the meantime,
    /// or if transactions have been removed from the mempool)
    ValidateTemplate(Block),
    /// If template is valid
    TemplateValidity(bool),
    /// Submit a mined block to a node
    SubmitTemplate(Block),
    /// Ask a node to report all the other nodes it knows
    /// about
    DiscoverNodes,
    /// This is the response to DiscoverNodes
    NodeList(Vec<String>),
    /// Ask a node whats the highest block it knows about
    /// in comparison to the local blockchain
    AskDifference(u32),
    /// This is the response to AskDifference
    Difference(i32),
    /// Ask a node to send a block with the specified height
    FetchBlock(usize),
    /// Broadcast a new block to other nodes
    NewBlock(Block),
}

impl Message {

    pub fn encode(self: &Self) -> Result<Vec<u8>, ciborium::ser::Error<IOError>> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes)?;
        return Ok(bytes);
    }

    pub fn decode(data: &[u8]) -> Result<Self, ciborium::de::Error<IOError>> {
        return ciborium::from_reader(data);
    }

    pub fn send(self: &Self, stream: &mut impl Write) -> Result<(), ciborium::ser::Error<IOError>> {
        let bytes = self.encode()?;
        let len = bytes.len() as u64;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&bytes)?;
        return Ok(());
    }

    pub fn recieve(stream: &mut impl Read) -> Result<Self, ciborium::de::Error<IOError>> {
        let mut len_bytes =  [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        let len = u64::from_be_bytes(len_bytes) as usize;
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data)?;
        return Self::decode(&data);
    }
}

