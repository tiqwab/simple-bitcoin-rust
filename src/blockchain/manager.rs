use crate::blockchain::block::{Block, BlockHash};
use crate::blockchain::transaction::Transaction;
use crate::blockchain::transaction_pool::TransactionPool;
use crate::blockchain::util;
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use sha2::{Digest, Sha256};

pub struct BlockchainManager {
    chain: Vec<Block>,
    difficulty: usize,
}

impl BlockchainManager {
    pub fn new(difficulty: usize) -> BlockchainManager {
        BlockchainManager {
            chain: vec![],
            difficulty,
        }
    }

    pub fn get_last_block_hash(&self) -> BlockHash {
        if let Some(block) = self.chain.last() {
            block.calculate_hash().unwrap()
        } else {
            self.get_genesis_block_hash()
        }
    }

    pub fn get_genesis_block_hash(&self) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(r#"{"message":"this_is_simple_bitcoin_genesis_block"}"#);
        util::to_hex(hasher.finalize().to_vec())
    }

    pub fn get_chain(&self) -> Vec<Block> {
        self.chain.clone()
    }

    pub fn get_difficulty(&self) -> usize {
        self.difficulty
    }

    pub fn add_new_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn is_valid_block(&self, block: &Block) -> Result<()> {
        // check prev_block_hash
        let last_block_hash = self.get_last_block_hash();
        if last_block_hash != block.get_prev_block_hash() {
            let err = anyhow!(
                "prev_block_hash of block is {}, but that of the last block in the main chain is {}. Block: {:?}",
                block.get_prev_block_hash(), last_block_hash, block,
            );
            return Err(err);
        }

        // check difficulty
        if !block.is_valid(self.difficulty)? {
            let err = anyhow!(
                "block hash ({}) doesn't satisfy difficulty ({}). Block: {:?}",
                block.calculate_hash()?,
                self.difficulty,
                block
            );
            return Err(err);
        }

        Ok(())
    }

    /// TransactionPool から自身の blockchain にすでに取り込んだ transaction を除く。
    /// 主に他 Core ノードから新 block を受け取った場合に必要な処理。
    pub fn remove_useless_transactions(&self, pool: &mut TransactionPool) {
        for block in self.chain.iter() {
            for transaction_in_block in block.get_transactions() {
                for transaction_in_pool in pool.get_transactions() {
                    if *transaction_in_block == transaction_in_pool {
                        pool.remove_transaction(&transaction_in_pool);
                    }
                }
            }
        }
    }

    /// 他 Core ノードから受け取った blockchain と比較して必要ならそれを main chain とする。
    /// その場合に除かれることになる block 内の未反映 transactions を返す。
    pub fn resolve_conflicts(&mut self, other_chain: Vec<Block>) -> Vec<Transaction> {
        if self.chain.len() >= other_chain.len() {
            warn!(
                "Received full chain is shorter than me, ignore it: {:?}",
                other_chain
            );
            return vec![];
        }

        if !is_valid_chain(self.get_genesis_block_hash(), &other_chain).unwrap() {
            warn!(
                "Received full chain is invalid, ignore it: {:?}",
                other_chain
            );
            return vec![];
        }

        let orphan_blocks = self
            .chain
            .iter()
            .cloned()
            .filter(|t| !other_chain.contains(t))
            .collect::<Vec<_>>();

        let orphan_transactions = orphan_blocks
            .into_iter()
            .flat_map(|b| b.get_transactions().to_owned())
            .collect::<Vec<_>>();

        self.chain = other_chain;

        let main_transactions = self
            .chain
            .iter()
            .flat_map(|b| b.get_transactions().to_owned())
            .collect::<Vec<_>>();

        orphan_transactions
            .into_iter()
            .filter(|t| !main_transactions.contains(&t))
            .collect()
    }
}

/// 渡された chain が valid か確認する
fn is_valid_chain(first_hash: BlockHash, chain: &Vec<Block>) -> Result<bool> {
    let mut prev_block_hash = first_hash;
    for block in chain.iter() {
        if prev_block_hash != block.get_prev_block_hash() {
            return Ok(false);
        }
        prev_block_hash = block.calculate_hash()?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::block::BlockWithoutProof;
    use crate::blockchain::transaction::Transaction;

    async fn generate_block(transactions: Vec<Transaction>, manager: &BlockchainManager) -> Block {
        BlockWithoutProof::new(transactions, manager.get_last_block_hash())
            .mine(manager.get_difficulty())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_remove_useless_transactions() {
        // setup
        let mut pool = TransactionPool::new();
        let mut manager = BlockchainManager::new(2);

        let block =
            generate_block(vec![Transaction::new("sender1", "recipient1", 1)], &manager).await;
        manager.add_new_block(block);

        let trans1 = Transaction::new("sender1", "recipient1", 1);
        let trans2 = Transaction::new("sender1", "recipient2", 1);
        pool.add_new_transaction(trans1.clone());
        pool.add_new_transaction(trans2.clone());

        // exercise
        manager.remove_useless_transactions(&mut pool);

        // verify
        assert_eq!(pool.get_transactions(), vec![trans2]);
    }

    #[tokio::test]
    async fn test_resolve_conflicts_longer_than_mine() {
        // setup
        let mut manager = BlockchainManager::new(2);

        // manager contains only block1 and block2
        let block1 =
            generate_block(vec![Transaction::new("sender1", "recipient1", 1)], &manager).await;
        manager.add_new_block(block1.clone());

        let block2 = generate_block(
            vec![
                Transaction::new("sender1", "recipient2", 1),
                Transaction::new("sender1", "recipient3", 1),
            ],
            &manager,
        )
        .await;
        manager.add_new_block(block2.clone());

        let block3 =
            generate_block(vec![Transaction::new("sender1", "recipient3", 1)], &manager).await;

        let block4 =
            generate_block(vec![Transaction::new("sender1", "recipient3", 1)], &manager).await;

        // exercise
        let other_chain = vec![block1, block3, block4];
        let res = manager.resolve_conflicts(other_chain.clone());

        // verify
        assert_eq!(res, vec![Transaction::new("sender1", "recipient2", 1)]);
        assert_eq!(manager.get_chain(), other_chain);
    }

    #[tokio::test]
    async fn test_resolve_conflicts_shorter_than_mine() {
        // setup
        let mut manager = BlockchainManager::new(2);

        let block1 =
            generate_block(vec![Transaction::new("sender1", "recipient1", 1)], &manager).await;
        manager.add_new_block(block1.clone());

        let block2 =
            generate_block(vec![Transaction::new("sender1", "recipient2", 1)], &manager).await;
        manager.add_new_block(block2.clone());

        // exercise
        let other_chain = vec![block1.clone()];
        let res = manager.resolve_conflicts(other_chain);

        // verify
        assert_eq!(res.len(), 0);
        assert_eq!(manager.get_chain(), vec![block1, block2]);
    }
}
