#[derive(Debug)]
pub enum Error
{
    OutOfBounds,
    InvalidHash,
    NetworkInfallible(String),
    NetworkMultiaddr(String),
    NetworkTransport(String),
    NetworkDial(String),
}

impl std::fmt::Display for Error
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::NetworkInfallible(err) => write!(fmt, "Network Infallible Error: {}", err),
            Self::NetworkMultiaddr(err) => write!(fmt, "Network Multiaddr Error: {}", err),
            Self::NetworkTransport(err) => write!(fmt, "Network Transport Error: {}", err),
            Self::NetworkDial(err) => write!(fmt, "Network Dial Error: {}", err),
            _ => write!(fmt, "{:?}", self),
        }
    }
}

impl From<std::convert::Infallible> for Error
{
    fn from(err: std::convert::Infallible) -> Self 
    {
        Self::NetworkInfallible(err.to_string())
    }
}

impl std::convert::From<libp2p::swarm::DialError> for Error
{
    fn from(err: libp2p::swarm::DialError) -> Self 
    {
        Self::NetworkDial(err.to_string())
    }
}  

impl From<libp2p::TransportError<std::io::Error>> for Error
{
    fn from(err: libp2p::TransportError<std::io::Error>) -> Self 
    {
        Self::NetworkTransport(err.to_string())
    }
}

impl From<libp2p::multiaddr::Error> for Error
{
    fn from(err: libp2p::multiaddr::Error) -> Self 
    {
        Self::NetworkMultiaddr(err.to_string())
    }
}
impl std::error::Error for Error{}
