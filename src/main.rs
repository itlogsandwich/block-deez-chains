use crate::block::BlockState;
mod block;

fn main() 
{
    println!("Deploying Blockchain...\n");
    let mut chain = BlockState::new();

    chain.create_genesis_block();
 
    chain.add_block(uuid::Uuid::new_v4(),String::from("BLOCKCHAIN IS COOL"));

    chain.add_block(uuid::Uuid::new_v4(),String::from("SINULOG HACKATHON 2025!"));

    for i in chain.blocks
    {
        println!("Blockchain {i}")
    }
}
