use crate::blockchain::block::{Block, BlockHash};
use crate::blockchain::transaction::{NormalTransaction, Transaction};
use crate::blockchain::transaction_pool::TransactionPool;
use crate::util;
use anyhow::{anyhow, bail, Result};
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
        util::bytes_to_hex(&hasher.finalize().to_vec())
    }

    pub fn get_chain(&self) -> Vec<Block> {
        self.chain.clone()
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        let mut res = vec![];
        for block in self.get_chain() {
            res.extend(block.get_transactions());
        }
        res
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
            for transaction_in_block in block.get_normal_transactions() {
                for transaction_in_pool in pool.get_transactions() {
                    if transaction_in_block == transaction_in_pool {
                        pool.remove_transaction(&transaction_in_pool);
                    }
                }
            }
        }
    }

    /// 他 Core ノードから受け取った blockchain と比較して必要ならそれを main chain とする。
    /// その場合に除かれることになる block 内の未反映 transactions を返す。
    pub fn resolve_conflicts(&mut self, other_chain: Vec<Block>) -> Vec<NormalTransaction> {
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
            .flat_map(|b| b.get_normal_transactions().to_owned())
            .collect::<Vec<_>>();

        self.chain = other_chain;

        let main_transactions = self
            .chain
            .iter()
            .flat_map(|b| b.get_normal_transactions().to_owned())
            .collect::<Vec<_>>();

        orphan_transactions
            .into_iter()
            .filter(|t| !main_transactions.contains(&t))
            .collect()
    }

    pub fn is_valid_transaction(&self, tx: &NormalTransaction) -> Result<()> {
        // block 内に組み込まれた transaction か
        fn does_exist_in_chain(target: &NormalTransaction, chain: &Vec<Block>) -> Result<()> {
            for input in target.get_inputs() {
                let tx_opt = chain
                    .iter()
                    .flat_map(|block| block.get_transactions())
                    .find(|tx| tx == input.get_transaction());

                if let None = tx_opt {
                    bail!("Invalid input is included in transaction (not exist in chain)");
                }
            }
            Ok(())
        }

        // まだ使われていない transaction か
        fn is_utxo(target: &NormalTransaction, chain: &Vec<Block>) -> Result<()> {
            for target_input in target.get_inputs() {
                let input_opt = chain
                    .iter()
                    .flat_map(|block| block.get_transactions())
                    .flat_map(|tx| tx.get_inputs())
                    .find(|input| input == &target_input);

                if let Some(_) = input_opt {
                    bail!("Invalid input is included in transaction (already used)");
                }
            }
            Ok(())
        }

        does_exist_in_chain(tx, &self.get_chain())?;
        is_utxo(tx, &self.get_chain())?;
        Ok(())
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
    use crate::blockchain::transaction::{
        CoinbaseTransaction, Transaction, TransactionInput, TransactionOutput, Transactions,
    };
    use crate::blockchain::utxo::UTXOManager;
    use crate::key_manager::KeyManager;
    use chrono::{Duration, Utc};
    use rand::rngs::OsRng;

    fn generate_block(
        transactions: Vec<NormalTransaction>,
        prev_block_hash: BlockHash,
        difficulty: usize,
    ) -> Block {
        let coinbase = CoinbaseTransaction::new("recipient1".to_string(), 10, Utc::now());
        BlockWithoutProof::new(Transactions::new(coinbase, transactions), prev_block_hash)
            .mine(difficulty)
            .unwrap()
    }

    #[test]
    fn test_remove_useless_transactions() {
        // setup
        let mut pool = TransactionPool::new();
        let mut manager = BlockchainManager::new(1);

        let base = Transaction::Coinbase(CoinbaseTransaction::new(
            "alice".to_string(),
            10,
            Utc::now(),
        ));

        let trans1 = NormalTransaction::new(
            vec![TransactionInput::new(base, 0)],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );
        let trans2 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Normal(trans1.clone()),
                0,
            )],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        let block = generate_block(
            vec![trans1.clone()],
            manager.get_last_block_hash(),
            manager.get_difficulty(),
        );
        manager.add_new_block(block);

        pool.add_new_transaction(trans1.clone());
        pool.add_new_transaction(trans2.clone());

        // exercise
        manager.remove_useless_transactions(&mut pool);

        // verify
        assert_eq!(pool.get_transactions(), vec![trans2]);
    }

    #[test]
    fn test_resolve_conflicts_longer_than_mine() {
        // setup
        let mut manager = BlockchainManager::new(1);

        let base = Transaction::Coinbase(CoinbaseTransaction::new(
            "alice".to_string(),
            10,
            Utc::now(),
        ));

        let trans1 = NormalTransaction::new(
            vec![TransactionInput::new(base, 0)],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        let trans2 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Normal(trans1.clone()),
                0,
            )],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        let trans3 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Normal(trans2.clone()),
                0,
            )],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        // manager contains only block1 and block2
        let block1 = generate_block(
            vec![trans1.clone()],
            manager.get_last_block_hash(),
            manager.get_difficulty(),
        );
        manager.add_new_block(block1.clone());

        let block2 = generate_block(
            vec![trans2.clone(), trans3.clone()],
            manager.get_last_block_hash(),
            manager.get_difficulty(),
        );
        manager.add_new_block(block2.clone());

        let block3 = generate_block(
            vec![trans3.clone()],
            block1.calculate_hash().unwrap(),
            manager.get_difficulty(),
        );

        let block4 = generate_block(
            vec![trans3.clone()],
            block3.calculate_hash().unwrap(),
            manager.get_difficulty(),
        );

        // exercise
        let other_chain = vec![block1, block3, block4];
        let res = manager.resolve_conflicts(other_chain.clone());

        // verify
        assert_eq!(res, vec![trans2]);
        assert_eq!(manager.get_chain(), other_chain);
    }

    #[test]
    fn test_resolve_conflicts_shorter_than_mine() {
        // setup
        let mut manager = BlockchainManager::new(1);

        let base = Transaction::Coinbase(CoinbaseTransaction::new(
            "alice".to_string(),
            10,
            Utc::now(),
        ));

        let trans1 = NormalTransaction::new(
            vec![TransactionInput::new(base, 0)],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        let trans2 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Normal(trans1.clone()),
                0,
            )],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        let trans3 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Normal(trans2.clone()),
                0,
            )],
            vec![TransactionOutput::new("recipient1".to_string(), 1)],
            Utc::now(),
        );

        let block1 = generate_block(
            vec![trans1.clone()],
            manager.get_last_block_hash(),
            manager.get_difficulty(),
        );
        manager.add_new_block(block1.clone());

        let block2 = generate_block(
            vec![trans2.clone(), trans3.clone()],
            manager.get_last_block_hash(),
            manager.get_difficulty(),
        );
        manager.add_new_block(block2.clone());

        // exercise
        let other_chain = vec![block1.clone()];
        let res = manager.resolve_conflicts(other_chain);

        // verify
        assert_eq!(res.len(), 0);
        assert_eq!(manager.get_chain(), vec![block1, block2]);
    }

    #[test]
    fn test_is_valid_transaction_returns_ok() {
        // setup
        let rng = OsRng;
        let km1 = KeyManager::new(rng.clone()).unwrap();
        let km2 = KeyManager::new(rng.clone()).unwrap();
        let mut um1 = UTXOManager::new(km1.get_address());
        let mut bm = BlockchainManager::new(1);

        // block1
        let tx1 = CoinbaseTransaction::new(km1.get_address(), 20, Utc::now());
        let block1 = BlockWithoutProof::new(
            Transactions::new(tx1.clone(), vec![]),
            bm.get_last_block_hash(),
        )
        .mine(bm.get_difficulty())
        .unwrap();
        bm.add_new_block(block1.clone());
        um1.refresh_utxos(&bm.get_transactions());

        // block2
        let tx2 = CoinbaseTransaction::new(km1.get_address(), 10, Utc::now());
        let tx3 = um1.create_transaction_for(km2.get_address(), 5, 1).unwrap();
        let block2 = BlockWithoutProof::new(
            Transactions::new(tx2.clone(), vec![tx3.clone()]),
            bm.get_last_block_hash(),
        )
        .mine(bm.get_difficulty())
        .unwrap();
        bm.add_new_block(block2.clone());
        um1.refresh_utxos(&bm.get_transactions());

        assert!(is_valid_chain(bm.get_genesis_block_hash(), &bm.get_chain()).unwrap());

        // exercise and verify
        let new_tx = NormalTransaction::new(
            vec![TransactionInput::new(Transaction::Normal(tx3.clone()), 0)],
            vec![TransactionOutput::new(km1.get_address(), 4)],
            Utc::now(),
        );
        assert!(bm.is_valid_transaction(&new_tx).is_ok());
    }

    #[test]
    fn test_is_valid_transaction_returns_err() {
        // setup
        let rng = OsRng;
        let km1 = KeyManager::new(rng.clone()).unwrap();
        let km2 = KeyManager::new(rng.clone()).unwrap();
        let km3 = KeyManager::new(rng.clone()).unwrap();
        let mut um1 = UTXOManager::new(km1.get_address());
        let mut bm = BlockchainManager::new(1);

        // block1
        let tx1 = CoinbaseTransaction::new(km1.get_address(), 20, Utc::now());
        let block1 = BlockWithoutProof::new(
            Transactions::new(tx1.clone(), vec![]),
            bm.get_last_block_hash(),
        )
        .mine(bm.get_difficulty())
        .unwrap();
        bm.add_new_block(block1.clone());
        um1.refresh_utxos(&bm.get_transactions());

        // block2
        let tx2 = CoinbaseTransaction::new(km1.get_address(), 10, Utc::now());
        let tx3 = um1.create_transaction_for(km2.get_address(), 5, 1).unwrap();
        let block2 = BlockWithoutProof::new(
            Transactions::new(tx2.clone(), vec![tx3]),
            bm.get_last_block_hash(),
        )
        .mine(bm.get_difficulty())
        .unwrap();
        bm.add_new_block(block2.clone());
        um1.refresh_utxos(&bm.get_transactions());

        assert!(is_valid_chain(bm.get_genesis_block_hash(), &bm.get_chain()).unwrap());

        // exercise and verify with unknown transaction
        let new_tx = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Coinbase(CoinbaseTransaction::new(km3.get_address(), 10, Utc::now())),
                0,
            )],
            vec![TransactionOutput::new(km1.get_address(), 4)],
            Utc::now(),
        );
        assert!(bm.is_valid_transaction(&new_tx).is_err());

        // exercise and verify with used transaction
        let new_tx = NormalTransaction::new(
            vec![TransactionInput::new(Transaction::Coinbase(tx1.clone()), 0)],
            vec![TransactionOutput::new(km1.get_address(), 4)],
            Utc::now(),
        );
        assert!(bm.is_valid_transaction(&new_tx).is_err());
    }
}
