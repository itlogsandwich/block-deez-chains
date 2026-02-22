use libp2p::{gossipsub, mdns, ping, swarm::NetworkBehaviour};

use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct AppBehaviour
{
    pub gossipsub: gossipsub::Behaviour,
    pub ping: ping::Behaviour,
}

#[derive(Debug)]
pub enum Event
{
    Gossipsub(gossipsub::Event),
    Ping(ping::Event),
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
