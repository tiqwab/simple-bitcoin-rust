use log::info;
use simple_bitcoin::blockchain::manager::BlockchainManager;
use simple_bitcoin::blockchain::transaction::Transaction;
use simple_bitcoin::blockchain::transaction_pool::TransactionPool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();

    let bm = Arc::new(Mutex::new(BlockchainManager::new()));
    let tp = Arc::new(Mutex::new(TransactionPool::new()));

    tokio::spawn(TransactionPool::generate_block_periodically(
        Arc::clone(&tp),
        Arc::clone(&bm),
        Duration::from_secs(10),
    ));

    let prev_block_hash = bm.lock().unwrap().get_genesis_block_hash();
    info!("genesis_block_hash: {:?}", prev_block_hash);

    let transaction1 = Transaction::new("test1", "test2", 1);
    tp.lock().unwrap().add_new_transaction(transaction1);

    let transaction2 = Transaction::new("test1", "test3", 2);
    tp.lock().unwrap().add_new_transaction(transaction2);

    tokio::time::sleep(Duration::from_secs(20)).await;

    let transaction3 = Transaction::new("test1", "test4", 3);
    tp.lock().unwrap().add_new_transaction(transaction3);

    tokio::time::sleep(Duration::from_secs(60)).await;
}
