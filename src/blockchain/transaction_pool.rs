use crate::blockchain::block::Block;
use crate::blockchain::manager::BlockchainManager;
use crate::blockchain::transaction::Transaction;
use log::debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct TransactionPool {
    transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> TransactionPool {
        TransactionPool {
            transactions: vec![],
        }
    }

    pub fn add_new_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn clear_transactions(&mut self) {
        self.transactions.clear();
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    #[async_recursion::async_recursion]
    pub async fn generate_block_periodically(
        pool: Arc<Mutex<TransactionPool>>,
        manager: Arc<Mutex<BlockchainManager>>,
        interval: Duration,
    ) {
        debug!("generate_block_periodically was called");

        {
            let mut pool = pool.lock().unwrap();
            let transactions = pool.get_transactions();

            if !transactions.is_empty() {
                let mut manager = manager.lock().unwrap();
                let prev_block_hash = manager.get_last_block_hash();
                let block = Block::new(transactions, prev_block_hash.clone());
                manager.add_new_block(block);

                pool.clear_transactions();

                debug!("Current blockchain is: {:?}", manager.get_chain());
            }
        }

        tokio::time::sleep(interval).await;
        Self::generate_block_periodically(pool, manager, interval).await;
    }
}
