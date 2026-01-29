use crate::block::BlockState;
use crate::error::Error;
mod block;
mod error;

use libp2p::{
    noise,
    tcp,
    yamux,
    ping,
    Multiaddr,
    futures::StreamExt,
};

#[tokio::main]
async fn main() -> Result<(), Error>
{
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        ).unwrap()
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| 
        {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(u64::MAX))
        })
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if let Some(addr) = std::env::args().nth(1) 
    {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

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

    loop
    {
        match swarm.select_next_some().await
        {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => 
            {
                println!("Listening on {address}")
            },
            libp2p::swarm::SwarmEvent::Behaviour(event) =>
            {
                println!("Event: {event:?}")
            },
            _ => {}
        }
    }

    Ok(())
}
