use serde::{ Serialize, Deserialize };
use chrono::Utc;
use sha2::{Sha256, Digest};
#[derive(Debug, Serialize, Deserialize)]
pub struct Block
{
    index: u64,
    timestamp: i64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

pub struct BlockState
{
    pub blocks: Vec<Block>,
}

impl BlockState
{

    fn new () -> Self
    {
        Self
        {
            blocks: Vec::new()
        }
    }

    fn create_genesis_block(&mut self)
    {
        let genesis_block = Block
        {
            index: 0,
            timestamp: Utc::now().timestamp(),
            data: String::from("DAPProptech is the way"),
            previous_hash: String::from("0"),
            hash: String::from("0000000000000000000000000000000000000000000000000000000000000000000000"),
            nonce: 3694,
        };
    
        self.blocks.push(genesis_block)
    }

    fn add_block(&mut self, index: u64, data: String, previous_hash: String)
    {
        let timestamp = Utc::now().timestamp();
        let nonce = 3694;
        let hash = calculate_hash(index, &data, &previous_hash, nonce);

        let block = Block
        {
            index,
            timestamp,
            data,
            previous_hash,
            hash,
            nonce,
        };

        self.blocks.push(block)
    }
}

fn calculate_hash(index: u64, data: &str, previous_hash: &str, nonce: u64) -> String
{
    let mut hasher = Sha256::new();

    let val = format!("{index}{data}{previous_hash}{nonce}");
    hasher.update(val);
    let hash = hasher.finalize();

    hex::encode(hash)
}
