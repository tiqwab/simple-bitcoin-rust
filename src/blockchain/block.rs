use crate::blockchain::transaction::{NormalTransaction, Transaction, Transactions};
use crate::blockchain::util;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type BlockHash = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockWithoutProof {
    timestamp: DateTime<Utc>,
    transaction: Transactions,
    prev_block_hash: BlockHash,
}

impl BlockWithoutProof {
    pub fn new(transaction: Transactions, prev_block_hash: BlockHash) -> BlockWithoutProof {
        BlockWithoutProof {
            timestamp: Utc::now(),
            transaction,
            prev_block_hash,
        }
    }

    pub fn get_prev_block_hash(&self) -> BlockHash {
        self.prev_block_hash.clone()
    }

    pub fn mine(self, difficulty: usize) -> Result<Block> {
        let mut nonce = 0;
        let target = "0".repeat(difficulty);
        loop {
            let hash = self.calculate_hash(nonce)?;
            if hash.ends_with(&target) {
                let block = Block::new(self, nonce);
                return Ok(block);
            }
            nonce += 1;
        }
    }

    fn calculate_hash(&self, nonce: usize) -> Result<BlockHash> {
        let data = serde_json::to_string(self)?;
        Ok(util::sha256(data.as_bytes(), &nonce.to_be_bytes()))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    #[serde(flatten)]
    inner: BlockWithoutProof,
    nonce: usize,
}

impl Block {
    pub fn new(inner: BlockWithoutProof, nonce: usize) -> Block {
        Block { inner, nonce }
    }

    pub fn get_normal_transactions(&self) -> Vec<NormalTransaction> {
        self.inner.transaction.get_normal_transactions()
    }

    pub fn get_prev_block_hash(&self) -> BlockHash {
        self.inner.prev_block_hash.clone()
    }

    // 常に生成される json のキー順が一致するかどうか確認が必要。
    // -> serde_json は どうやら struct で定義した順に出力する。
    // https://users.rust-lang.org/t/order-of-fields-in-serde-json-to-string/48928
    pub fn calculate_hash(&self) -> Result<BlockHash> {
        self.inner.calculate_hash(self.nonce)
    }

    pub fn is_valid(&self, difficulty: usize) -> Result<bool> {
        let hash = self.calculate_hash()?;
        let target = "0".repeat(difficulty);
        Ok(hash.ends_with(&target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::transaction::CoinbaseTransaction;

    #[tokio::test]
    async fn test_block_mine() {
        let block_without_proof = BlockWithoutProof::new(
            Transactions::new(
                CoinbaseTransaction::new("recipient1".to_string(), 2),
                vec![],
            ),
            util::sha256("foo".as_bytes(), "123".as_bytes()),
        );

        let block = block_without_proof.mine(4).unwrap();
        assert!(block.calculate_hash().unwrap().ends_with("0000"));
    }
}
