use libp2p::{ noise,
    tcp,
    yamux,
    ping,
    gossipsub,
    Multiaddr,
    futures::StreamExt,
    gossipsub::{MessageAuthenticity, IdentTopic},
    swarm::SwarmEvent,
};
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::Utc;

use crate::block::{BlockState, Block, BlockCandidate, mine_block};
use crate::error::Error;
use crate::p2p::AppBehaviour;
use crate::p2p::Event as MainEvent;

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

    let (tx, mut rx) = mpsc::channel::<Block>(100);
    
    let miner_tx = tx.clone();
    let last_block = chain.blocks.last().unwrap().clone();
    
    tokio::task::spawn_blocking(move || 
    {
            let candidate = BlockCandidate
            {
                index: Uuid::new_v4(),
                timestamp: Utc::now().timestamp(),
                data: String::from("Test"),
                previous_hash: last_block.hash, 
            };
            
            let mined = mine_block(candidate);
            _ = miner_tx.blocking_send(mined);
    });

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
                            let incoming_block: Block = block;
                            
                            if let Err(e) = chain.add_block(incoming_block)
                            {
                                println!("An error has occured! {e}");
                            }
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
            },
            Some(new_block) = rx.recv() =>
            {
                println!("Miner has found a new block! {:?}", &new_block.hash);
                
                if let Ok(serialized_block) = serde_json::to_vec(&new_block)
                {
                    match chain.add_block(new_block)
                    {
                        Ok(()) =>
                        {
                            println!("Block found! Adding...");
                            _ = swarm.behaviour_mut().gossipsub.publish(topic.hash(), serialized_block);
                        }
                        Err(e) => println!("An error has occured! {e}"),
                    }
                }
                else
                {
                    println!("Data Serialization failed...");
                }
            }
        }
    }
    Ok(())
}
