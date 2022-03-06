use crate::blockchain::block::BlockWithoutProof;
use crate::blockchain::manager::BlockchainManager;
use crate::blockchain::transaction::Transaction;
use crate::connection_manager_core::{ConnectionManagerCore, ConnectionManagerInner};
use crate::message::{ApplicationPayload, Message, Payload};
use log::{debug, info};
use std::ops::RangeBounds;
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
        if !self.has_transaction(&transaction) {
            self.transactions.push(transaction);
        }
    }

    pub fn has_transaction(&self, transaction: &Transaction) -> bool {
        self.transactions.contains(transaction)
    }

    pub fn clear_transactions(&mut self) {
        self.transactions.clear();
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    pub fn take_transactions(&mut self) -> Vec<Transaction> {
        self.transactions.drain(0..).collect()
    }

    pub fn remove_transactions<R: RangeBounds<usize>>(&mut self, range: R) {
        self.transactions.drain(range);
    }

    pub fn remove_transaction(&mut self, transaction: &Transaction) {
        if let Some((index, _)) = self
            .transactions
            .iter()
            .enumerate()
            .find(|(_, t)| *t == transaction)
        {
            self.transactions.remove(index);
        }
    }

    #[async_recursion::async_recursion]
    pub async fn generate_block_periodically(
        pool: Arc<Mutex<TransactionPool>>,
        blockchain_manager: Arc<Mutex<BlockchainManager>>,
        connection_manager: Arc<Mutex<ConnectionManagerInner>>,
        interval: Duration,
    ) {
        debug!("generate_block_periodically was called");

        let transactions = pool.lock().unwrap().get_transactions();
        let num_transactions = transactions.len();

        let difficulty = blockchain_manager.lock().unwrap().get_difficulty();

        if !transactions.is_empty() {
            let prev_block_hash = blockchain_manager.lock().unwrap().get_last_block_hash();
            let block = tokio::task::spawn_blocking(move || {
                BlockWithoutProof::new(transactions, prev_block_hash.clone()).mine(difficulty)
            })
            .await
            .unwrap()
            .unwrap();

            let is_block_added;
            {
                let mut manager = blockchain_manager.lock().unwrap();
                let mut pool = pool.lock().unwrap();

                // If new blocks came from other core nodes, throw away the block and do again
                is_block_added = block.get_prev_block_hash() != manager.get_last_block_hash();
                if is_block_added {
                    info!("generated block, but it was old. Ignore it.");
                } else {
                    manager.add_new_block(block.clone());
                    debug!("generated block: {:?}", block);
                    debug!("Current blockchain is: {:?}", manager.get_chain());
                    pool.remove_transactions(0..num_transactions);
                }
            }

            // notify a new block
            if !is_block_added {
                let payload = Payload::Application {
                    payload: ApplicationPayload::NewBlock { block },
                };
                let port = connection_manager.lock().unwrap().get_my_addr().port();
                ConnectionManagerCore::send_msg_to_core_nodes(
                    Arc::clone(&connection_manager),
                    Message::new(port, payload),
                )
                .await;
            }
        }

        tokio::time::sleep(interval).await;
        Self::generate_block_periodically(pool, blockchain_manager, connection_manager, interval)
            .await;
    }
}