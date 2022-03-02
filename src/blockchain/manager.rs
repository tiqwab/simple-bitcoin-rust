use crate::blockchain::block::{Block, BlockHash};
use anyhow::Result;
use sha2::{Digest, Sha256};

fn to_hex(xs: Vec<u8>) -> String {
    fn hex(x: u8) -> [char; 2] {
        let chars = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
        ];
        let mut buf = ['\0'; 2];
        buf[0] = chars[(x / 16) as usize];
        buf[1] = chars[(x % 16) as usize];
        buf
    }

    xs.into_iter().map(|x| hex(x)).flatten().collect()
}

pub struct BlockchainManager {
    chain: Vec<Block>,
}

impl BlockchainManager {
    pub fn new() -> BlockchainManager {
        BlockchainManager { chain: vec![] }
    }

    // 常に生成される json のキー順が一致するかどうか確認が必要。
    // -> serde_json は どうやら struct で定義した順に出力する。
    // https://users.rust-lang.org/t/order-of-fields-in-serde-json-to-string/48928
    pub fn get_hash(&self, block: &Block) -> Result<BlockHash> {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(block)?.as_bytes());
        Ok(to_hex(hasher.finalize().to_vec()))
    }

    pub fn get_genesis_block_hash(&self) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(r#"{"message":"this_is_simple_bitcoin_genesis_block"}"#);
        to_hex(hasher.finalize().to_vec())
    }

    pub fn add_new_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn is_valid_chain(&self) -> Result<bool> {
        let mut prev_block_hash = self.get_genesis_block_hash();
        for block in self.chain.iter() {
            if prev_block_hash != block.get_prev_block_hash() {
                return Ok(false);
            }
            prev_block_hash = self.get_hash(block)?;
        }
        Ok(true)
    }
}
