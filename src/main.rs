use libp2p::{ noise,
    tcp,
    yamux,
    ping,
    gossipsub,
    Multiaddr,
    futures::StreamExt,
    gossipsub::{Event, MessageAuthenticity, IdentTopic},
    swarm::SwarmEvent,
};
use tokio::sync::mpsc;

use crate::block::BlockState;
use crate::error::Error;
use crate::p2p::AppBehaviour;
use crate::p2p::Event as MainEvent;
use crate::block::Block;

mod block;
mod error;
mod p2p;

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
        .with_behaviour(|key| 
        {
            let gossipsub = gossipsub::Behaviour::new(MessageAuthenticity::Signed(key.clone()),gossipsub::Config::default()).expect("Gossipsub failed");
            AppBehaviour
            {
                gossipsub,
                ping: ping::Behaviour::default(),
            }
        })?
        .with_swarm_config(|cfg| 
        {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(u64::MAX))
        })
        .build();
    
    let topic = IdentTopic::new("Blockchain");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).expect("Topic subscription failed");



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
        tokio::select!
        {
            event = swarm.select_next_some() =>
            {
                match event
                {
                    SwarmEvent::Behaviour(MainEvent::Gossipsub(gossipsub::Event::Message{ message,.. })) => 
                    {
                        if let Ok(block) = serde_json::from_slice(&message.data)
                        {
                            let _incoming_block: Block = block;
                        }
                        else
                        {
                            println!("Data lost in transmission...");
                        }
                        println!("Message Received! {:?}", message.data);
                    },
                    SwarmEvent::Behaviour(MainEvent::Ping(ping_event)) => 
                    { 
                        println!("Pinging! {:?}", ping_event); 
                    },

                    _ => {}
                }
            }
        }
    }
    Ok(())
}
