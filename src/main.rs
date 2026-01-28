use crate::block::BlockState;
use crate::error::Error;
mod block;
mod error;

fn main() -> Result<(), Error>
{
    println!("Deploying Blockchain...\n");
    let mut chain = BlockState::new();

    chain.create_genesis_block();

    let mined = chain.mine_block(chain.blocks.last().unwrap().index, chain.blocks.last().unwrap().previous_hash.as_str(), chain.blocks.last().unwrap().hash.as_str());
    chain.add_block(String::from("BLOCKCHAIN IS COOL"), mined.0, mined.1)?;

    let mined = chain.mine_block(chain.blocks.last().unwrap().index, chain.blocks.last().unwrap().previous_hash.as_str(), chain.blocks.last().unwrap().hash.as_str());
    chain.add_block(String::from("SINULOG HACKATHON 2025!"), mined.0, mined.1)?;

    for i in chain.blocks
    {
        println!("Blockchain {i}")
    }

    Ok(())
}
