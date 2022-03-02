use simple_bitcoin::blockchain::block::Block;
use simple_bitcoin::blockchain::manager::BlockchainManager;
use simple_bitcoin::blockchain::transaction::Transaction;

fn main() {
    let mut bm = BlockchainManager::new();

    let prev_block_hash = bm.get_genesis_block_hash();
    println!("genesis_block_hash: {:?}", prev_block_hash);

    let transaction1 = Transaction::new("test1", "test2", 3);
    let new_block1 = Block::new(transaction1, prev_block_hash);
    let new_block_hash1 = bm.get_hash(&new_block1).unwrap();
    println!("1st block_hash: {:?}", new_block_hash1);
    bm.add_new_block(new_block1);

    let transaction2 = Transaction::new("test1", "test3", 2);
    let new_block2 = Block::new(transaction2, new_block_hash1);
    let new_block_hash2 = bm.get_hash(&new_block2).unwrap();
    println!("2nd block_hash: {:?}", new_block_hash2);
    bm.add_new_block(new_block2);

    println!("{}", bm.is_valid_chain().unwrap())
}
