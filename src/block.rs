use crate::error::Error;
use std::fmt;
use serde::{ Serialize, Deserialize };
use chrono::Utc;
use sha2::{Sha256, Digest};
use uuid::Uuid;

const DEFAULT_PREFIX: &str = "6767";

type BlockResult<T> = Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block
{
    pub index: Uuid,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockCandidate
{
    pub index: Uuid,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
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

    pub fn add_block(&mut self, block: Block) -> BlockResult<()>
    {
        if self.blocks.last().is_none() 
        {
            return Err(Error::OutOfBounds);
        }

        let previous_hash = &self.blocks.last().unwrap().hash;

        self.compare_hash(previous_hash)?;

        check_prefix(block.index, &block.data, &block.hash, &block.previous_hash, block.nonce)?;

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

}

pub fn check_prefix(index: Uuid, data: &str, hash: &str, previous_hash: &str, nonce: u64) -> BlockResult<()>
{
    if hash.starts_with(DEFAULT_PREFIX)
    {
        let recalc_hash = calculate_hash(index, data, previous_hash, nonce);
        
        if recalc_hash != hash
        {
            return Err(Error::InvalidHash);
        }

        Ok(())
    }
    else
    {
        Err(Error::InvalidPrefix)
    }
}

pub fn mine_block(block_candidate: BlockCandidate) -> Block
{
    println!("Mining block...");
    let mut nonce = 0;

    loop
    {
        if nonce % 10000 == 0
        {
            println!("Nonce: {nonce}");
        }

        let hash = calculate_hash(block_candidate.index, &block_candidate.data, &block_candidate.previous_hash, nonce);
        if hash.starts_with(DEFAULT_PREFIX)
        {
            println!("
            Nonce: {nonce},
            Hash: {hash},
            ");

            return Block
            {
                index: block_candidate.index,
                timestamp: block_candidate.timestamp,
                data: block_candidate.data,
                previous_hash: block_candidate.previous_hash.to_string(),
                hash,
                nonce,
            }
        }            
        nonce += 1;
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
