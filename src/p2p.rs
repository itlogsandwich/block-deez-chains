use libp2p::{gossipsub, mdns, ping,request_response,swarm::NetworkBehaviour};
use serde::{Deserialize, Serialize};

use crate::block::Block;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct AppBehaviour
{
    pub gossipsub: gossipsub::Behaviour,
    pub ping: ping::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub request_response: request_response::json::Behaviour<BlockRequest, BlockResponse>,
}

#[derive(Debug)]
pub enum Event
{
    Gossipsub(gossipsub::Event),
    Ping(ping::Event),
    Mdns(mdns::Event),
    RequestResponse(request_response::Event<BlockRequest, BlockResponse>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockRequest
{
    GetBlock(u64),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BlockResponse
{
    FoundBlock(Block),
    BlockNotFound(u64), //404
}

impl From<gossipsub::Event> for Event
{
    fn from(event: gossipsub::Event) -> Self
    {
        Self::Gossipsub(event)
    }
}

impl From<ping::Event> for Event
{
    fn from(event: ping::Event) -> Self
    {
        Self::Ping(event)
    }
}

impl From<mdns::Event> for Event
{
    fn from(event: mdns::Event) -> Self 
    {
        Self::Mdns(event)
    }
}

impl From<request_response::Event<BlockRequest, BlockResponse>> for Event
{
    fn from(event: request_response::Event<BlockRequest, BlockResponse>) -> Self
    {
        Self::RequestResponse(event)
    }
}
