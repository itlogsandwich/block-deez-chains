use serde::{ Serialize, Deserialize };
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block
{
    index: u64,
    timestamp: i128,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block
{
    fn new(index: u64, data: String, previous_hash: String) -> Self
    {
        let timestamp = Utc::now();
        let nonce = 3694;
        let hash = calculate_hash(index, &data, &previous_hash, nonce);
        Self
        {
            index,
            timestamp,
            data,
            previous_hash,
            hash,
            nonce,
        }
    }
}

fn calculate_hash(index: u64, data: &str, previous_hash: &str, nonce: u64) -> String
{
    let mut hash = String::new();
    let val = index + data.bytes() + previous_hash.bytes() + nonce;

    hash = val.to_string();

    hash
}
