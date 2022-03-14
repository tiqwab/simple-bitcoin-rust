use crate::blockchain::block::BlockWithoutProof;
use crate::blockchain::manager::BlockchainManager;
use crate::blockchain::transaction::{
    CoinbaseTransaction, NormalTransaction, TransactionInput, Transactions,
};
use crate::connection_manager_core::{ConnectionManagerCore, ConnectionManagerInner};
use crate::key_manager::KeyManager;
use crate::message::{ApplicationPayload, Message, Payload};
use chrono::Utc;
use log::{debug, info};
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// FIXME: mining 報酬のベタ書き
pub const COINBASE_INCENTIVE: u64 = 10;

pub struct TransactionPool {
    transactions: Vec<NormalTransaction>,
}

impl TransactionPool {
    pub fn new() -> TransactionPool {
        TransactionPool {
            transactions: vec![],
        }
    }

    pub fn add_new_transaction(&mut self, transaction: NormalTransaction) {
        if !self.has_transaction(&transaction) {
            self.transactions.push(transaction);
        }
    }

    pub fn has_transaction(&self, transaction: &NormalTransaction) -> bool {
        self.transactions.contains(transaction)
    }

    pub fn clear_transactions(&mut self) {
        self.transactions.clear();
    }

    pub fn get_transactions(&self) -> Vec<NormalTransaction> {
        self.transactions.clone()
    }

    pub fn take_transactions(&mut self) -> Vec<NormalTransaction> {
        self.transactions.drain(0..).collect()
    }

    pub fn remove_transactions<R: RangeBounds<usize>>(&mut self, range: R) {
        self.transactions.drain(range);
    }

    pub fn remove_transaction(&mut self, transaction: &NormalTransaction) {
        if let Some((index, _)) = self
            .transactions
            .iter()
            .enumerate()
            .find(|(_, t)| *t == transaction)
        {
            self.transactions.remove(index);
        }
    }

    pub fn has_transaction_input(&self, target_input: &TransactionInput) -> bool {
        for tx in self.transactions.iter() {
            for input in tx.get_inputs() {
                if &input == target_input {
                    return true;
                }
            }
        }
        false
    }

    pub fn calc_total_fee(&self) -> u64 {
        self.transactions.iter().fold(0, |acc, tx| {
            let fee = tx.get_input_value() - tx.get_output_value();
            acc + fee
        })
    }

    #[async_recursion::async_recursion]
    pub async fn generate_block_periodically(
        pool: Arc<Mutex<TransactionPool>>,
        blockchain_manager: Arc<Mutex<BlockchainManager>>,
        connection_manager: Arc<Mutex<ConnectionManagerInner>>,
        key_manager: Arc<Mutex<KeyManager>>,
        interval: Duration,
    ) {
        debug!("generate_block_periodically was called");

        let pool_txs: Vec<NormalTransaction>;
        let num_pool_txs: usize;
        let total_fee: u64;
        {
            let pool = pool.lock().unwrap();
            pool_txs = pool.get_transactions();
            num_pool_txs = pool_txs.len();
            total_fee = pool.calc_total_fee();
        }

        let difficulty = blockchain_manager.lock().unwrap().get_difficulty();
        let addr = key_manager.lock().unwrap().get_address();

        let prev_block_hash = blockchain_manager.lock().unwrap().get_last_block_hash();
        let block = tokio::task::spawn_blocking(move || {
            let transactions = Transactions::new(
                CoinbaseTransaction::new(addr, COINBASE_INCENTIVE + total_fee, Utc::now()),
                pool_txs,
            );
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
                pool.remove_transactions(0..num_pool_txs);
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

        tokio::time::sleep(interval).await;
        Self::generate_block_periodically(
            pool,
            blockchain_manager,
            connection_manager,
            key_manager,
            interval,
        )
        .await;
    }
}
