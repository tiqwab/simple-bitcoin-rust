use crate::blockchain::transaction::Transaction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type BlockHash = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    timestamp: DateTime<Utc>,
    transaction: Vec<Transaction>,
    prev_block_hash: BlockHash,
}

impl Block {
    pub fn new(transaction: Vec<Transaction>, prev_block_hash: BlockHash) -> Block {
        Block {
            timestamp: Utc::now(),
            transaction,
            prev_block_hash,
        }
    }

    pub fn get_prev_block_hash(&self) -> BlockHash {
        self.prev_block_hash.clone()
    }
}
