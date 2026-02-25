use libp2p::{ noise,
    tcp,
    yamux,
    ping,
    gossipsub,
    Multiaddr,
    futures::StreamExt,
    gossipsub::{MessageAuthenticity, IdentTopic},
    swarm::SwarmEvent,
    mdns,
    request_response,
};
use tokio::sync::mpsc;
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};

use crate::block::{BlockState, Block, BlockCandidate, mine_block, mine_trigger};
use crate::error::Error;
use crate::p2p::{AppBehaviour, Event as MainEvent, BlockRequest, BlockResponse};

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

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id()).expect("Mdns failed");
            
            let protocol = [(libp2p::StreamProtocol::new("/blockchain-sync/v1"), request_response::ProtocolSupport::Full)];

            let req_resp = request_response::json::Behaviour::<BlockRequest, BlockResponse>::new(
                    protocol,
                    request_response::Config::default(),
                );
            AppBehaviour
            {
                gossipsub,
                ping: ping::Behaviour::default(),
                mdns,
                request_response: req_resp,

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
    let mut stop_signal = Arc::new(AtomicBool::new(false)); 
    
    mine_trigger(&chain, tx.clone(), stop_signal.clone());
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

                            match chain.add_block(incoming_block)
                            {
                                Ok(()) =>
                                {
                                    println!("Mining...");
                                    stop_signal = signal_control(stop_signal);
                                    mine_trigger(&chain, tx.clone(), stop_signal.clone());
                                },
                                Err(e) =>
                                {
                                    println!("An error has occured! {e}");

                                    if let Some(sender_peer_id) = message.source
                                    {
                                        let missing_height = chain.blocks.len() as u64;

                                        swarm.behaviour_mut().request_response.send_request(
                                                &sender_peer_id, 
                                                BlockRequest::GetBlock(missing_height),
                                            );
                                    }
                                }
                            };

                        }
                        else
                        {
                            println!("Data lost in transmission...");
                        }
                    },
                    SwarmEvent::Behaviour(MainEvent::Ping(ping_event)) => 
                    { 
                        println!("Pinging! {:?}", ping_event); 
                    },
                    SwarmEvent::Behaviour(MainEvent::Mdns(mdns_event)) =>
                    {
                        //For future reference, this event contains a vector of (peer_id, multiaddr)
                        match mdns_event
                        {
                            mdns::Event::Discovered(list) => 
                            {
                                println!("Discovering...");
                                for (peer_id, _ ) in list
                                {
                                    println!("mDNS discovered a new peer! {peer_id}"); 
                                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                                }
                            }
                            mdns::Event::Expired(list) => 
                            {
                                println!("Expired...");
                                for (peer_id, _) in list
                                {
                                    println!("mDNS peer has expired...{peer_id}");
                                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                                }
                            }
                        }
                    },
                    SwarmEvent::Behaviour(MainEvent::RequestResponse(request_response::Event::Message { peer, message, connection_id})) =>
                    {
                        match message 
                        {
                            request_response::Message::Request { request, channel, .. } =>
                            {
                                match request
                                {
                                    BlockRequest::GetBlock(height) =>
                                    {
                                        let response = match chain.blocks.get(height as usize)
                                        {
                                            Some(block) => BlockResponse::FoundBlock(block.clone()),
                                            None => BlockResponse::BlockNotFound(height),
                                        };

                                        swarm.behaviour_mut().request_response.send_response(channel, response).expect("Failed to send response");
                                    }

                                }
                            }

                            request_response::Message::Response { response, .. } =>
                            {
                                match response
                                {
                                    BlockResponse::FoundBlock(block) => 
                                    {
                                        println!("Received response, Adding block!");
                                        match chain.add_block(block)
                                        {
                                            Ok(()) =>
                                            {
                                                println!("Mining...");

                                                stop_signal = signal_control(stop_signal);
                                                mine_trigger(&chain, tx.clone(), stop_signal.clone());

                                                let next_height = chain.blocks.len() as u64;

                                                swarm.behaviour_mut().request_response.send_request(&peer, BlockRequest::GetBlock(next_height));
                                            },
                                            Err(e) => println!("An error has occured! {e}"),
                                        };
                                    },

                                    BlockResponse::BlockNotFound(height) =>
                                    {
                                        println!("Not found at height {height}");
                                    }
                                }
                            }

                        }
                    }

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
                            stop_signal = signal_control(stop_signal);
                            mine_trigger(&chain, tx.clone(), stop_signal.clone());
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

fn signal_control(mut stop_signal: Arc<AtomicBool>) -> Arc<AtomicBool>
{
    stop_signal.store(true, Ordering::SeqCst);

    stop_signal = Arc::new(AtomicBool::new(false));
    
    stop_signal
}
