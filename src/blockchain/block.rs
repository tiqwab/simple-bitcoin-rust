use crate::blockchain::transaction::{
    Address, CoinbaseTransaction, NormalTransaction, Transaction, Transactions,
};
use crate::blockchain::transaction_pool::COINBASE_INCENTIVE;
use crate::util;
use anyhow::{bail, Result};
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

    pub fn get_transaction_at(&self, idx: usize) -> Option<Transaction> {
        self.inner.transaction.get_transaction_at(idx)
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.inner.transaction.get_transactions()
    }

    pub fn get_coinbase_transaction(&self) -> CoinbaseTransaction {
        self.inner.transaction.get_coinbase_transaction()
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

    pub fn is_valid(&self, difficulty: usize) -> Result<()> {
        // check target
        let hash = self.calculate_hash()?;
        let target = "0".repeat(difficulty);
        if !hash.ends_with(&target) {
            bail!("invalid target");
        }

        // check incentive
        let coinbase_tx = self.get_coinbase_transaction();
        let normal_txs = self.get_normal_transactions();
        let mut total_fee = 0;
        for tx in normal_txs {
            total_fee += tx.get_input_value() - tx.get_output_value();
        }
        if coinbase_tx.get_value() != total_fee + COINBASE_INCENTIVE {
            bail!("invalid coinbase value");
        }

        Ok(())
    }

    pub fn miner(&self) -> Address {
        let tx = self.get_transaction_at(0).unwrap();
        tx.get_output(0).unwrap().get_recipient()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::transaction::{
        CoinbaseTransaction, TransactionInput, TransactionOutput,
    };
    use crate::blockchain::transaction_pool::COINBASE_INCENTIVE;
    use chrono::Duration;
    use std::str::FromStr;

    fn generate_sample(
        incentive: Option<u64>,
    ) -> (CoinbaseTransaction, NormalTransaction, Transactions) {
        let now: DateTime<Utc> = DateTime::from_str("2022-03-09T12:00:00Z").unwrap();
        let sec = Duration::seconds(1);

        let incentive = incentive.unwrap_or(COINBASE_INCENTIVE + 1);

        let tx1 = CoinbaseTransaction::new("alice".to_string(), incentive, now + sec * 0);
        let tx2 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Coinbase(CoinbaseTransaction::new(
                    "alice".to_string(),
                    10,
                    now + sec * 1,
                )),
                0,
            )],
            vec![TransactionOutput::new("bob".to_string(), 9)],
            Utc::now(),
        );
        let txs = Transactions::new(tx1.clone(), vec![tx2.clone()]);
        (tx1, tx2, txs)
    }

    #[tokio::test]
    async fn test_block_mine() {
        let (_, _, txs) = generate_sample(None);
        let block_without_proof =
            BlockWithoutProof::new(txs, util::sha256("foo".as_bytes(), "123".as_bytes()));

        let difficulty = 2;
        let block = block_without_proof.mine(difficulty).unwrap();
        assert!(block.calculate_hash().unwrap().ends_with("00"));
        assert!(block.is_valid(difficulty).is_ok())
    }

    #[tokio::test]
    async fn test_is_valid_returns_false_with_excessive_incentive() {
        let (_, _, txs) = generate_sample(Some(12));
        let block_without_proof =
            BlockWithoutProof::new(txs, util::sha256("foo".as_bytes(), "123".as_bytes()));

        let difficulty = 2;
        let block = block_without_proof.mine(difficulty).unwrap();
        assert!(block.is_valid(difficulty).is_err())
    }
}
