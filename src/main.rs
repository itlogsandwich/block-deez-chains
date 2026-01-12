use crate::block::BlockState;
mod block;

fn main() 
{
    println!("Deploying Blockchain...\n");
    let mut chain = BlockState::new();

    chain.create_genesis_block();
 
    chain.add_block(1, String::from("BLOCK CHAIN IS COOL"), chain.blocks[0].hash.to_string());

    for i in chain.blocks
    {
        println!("Blockchain {i}")
    }
}
