use crate::blockchain::transaction::Transaction;
use serde::{Deserialize, Serialize};

pub type BlockHash = String;

#[derive(Serialize, Deserialize)]
pub struct Block {
    transaction: Transaction,
    prev_block_hash: BlockHash,
}

impl Block {
    pub fn new(transaction: Transaction, prev_block_hash: BlockHash) -> Block {
        Block {
            transaction,
            prev_block_hash,
        }
    }

    pub fn get_prev_block_hash(&self) -> BlockHash {
        self.prev_block_hash.clone()
    }
}
