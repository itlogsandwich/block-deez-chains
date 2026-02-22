use crate::error::Error;
use std::fmt;
use serde::{ Serialize, Deserialize };
use serde_json::json;
use chrono::Utc;
use sha2::{Sha256, Digest};
use uuid::Uuid;

const DEFAULT_PREFIX: &str = "6767";

type BlockResult<T> = Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block
{
    pub index: Uuid,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl std::fmt::Display for Block
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "Index: {}\nTimestamp: {}\nData: {}\nPrevious Hash: {}\nCurrent Hash: {} \n", self.index, self.timestamp, self.data, self.previous_hash,self.hash)
    }
}

pub struct BlockState
{
    pub blocks: Vec<Block>,
}

impl BlockState
{
    pub fn new () -> Self
    {
        Self
        {
            blocks: Vec::new()
        }
    }

    pub fn create_genesis_block(&mut self)
    {
        let genesis_block = Block
        {
            index: Uuid::new_v4(),
            timestamp: Utc::now().timestamp(),
            data: String::from("DAPProptech is the way"),
            previous_hash: String::from("0"),
            hash: DEFAULT_PREFIX.to_owned() + "00000000000000000000000000000000000000000000000000000000000",
            nonce: 3694,
        };
    
        self.blocks.push(genesis_block)
    }

    pub fn add_block(&mut self, data: String, nonce: u64, hash: String) -> BlockResult<()>
    {
        if self.blocks.last().is_none() 
        {
            return Err(Error::OutOfBounds);
        }
        let index = Uuid::new_v4();
        let previous_hash = &self.blocks.last().unwrap().hash;

        self.compare_hash(previous_hash)?;

        let timestamp = Utc::now().timestamp();

        let block = Block
        {
            index,
            timestamp,
            data,
            previous_hash: previous_hash.to_string(),
            hash,
            nonce,
        };
        
        self.blocks.push(block);

        Ok(())
    }

    pub fn compare_hash(&self, hash: &str) -> BlockResult<()>
    {
        if self.blocks.last().unwrap().hash != hash
        {
            return Err(Error::InvalidHash);
        }

        Ok(())
    }

    pub fn mine_block(&self, index: Uuid, data: &str, previous_hash: &str) -> (u64, String)
    {
        println!("Mining block...");
        let mut nonce = 0;

        loop
        {
            if nonce % 10000 == 0
            {
                println!("Nonce: {nonce}");
            }

            let hash = calculate_hash(index, data, previous_hash, nonce);
            if hash.starts_with(DEFAULT_PREFIX)
            {
                println!("
                Nonce: {nonce},
                Hash: {hash},
                ");

                return (nonce, hash);
            }            
            nonce += 1;
        }
    }
}

fn calculate_hash(index: Uuid, data: &str, previous_hash: &str, nonce: u64) -> String
{
    let mut hasher = Sha256::new();

    let val = format!("{index}{data}{previous_hash}{nonce}");
    hasher.update(val);
    let hash = hasher.finalize();

    hex::encode(hash)
}
