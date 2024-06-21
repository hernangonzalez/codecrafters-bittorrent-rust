use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    fmt::Display,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Peer(Ipv4Addr, u16);

impl FromStr for Peer {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let (ip, port) = s.split_once(':').context("Not a Peer string")?;
        let ip = ip.parse::<Ipv4Addr>()?;
        let port = port.parse::<u16>()?;
        Ok(Self(ip, port))
    }
}

impl Display for Peer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl TryFrom<&[u8]> for Peer {
    type Error = anyhow::Error;
    fn try_from(bytes: &[u8]) -> Result<Self> {
        let bytes: &[u8; 6] = bytes.try_into()?;
        let ip = Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]);
        let port = u16::from_be_bytes([bytes[4], bytes[5]]);
        Ok(Self(ip, port))
    }
}

impl From<Peer> for SocketAddrV4 {
    fn from(p: Peer) -> Self {
        SocketAddrV4::new(p.0, p.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_display() {
        let p = Peer(Ipv4Addr::new(1, 2, 3, 4), u16::MAX);
        assert_eq!(p.to_string(), "1.2.3.4:65535")
    }

    #[test]
    fn peer_fromstr() {
        let p: Peer = "1.2.3.4:65535".parse().unwrap();
        assert_eq!(p.0, Ipv4Addr::new(1, 2, 3, 4));
        assert_eq!(p.1, 65535);
    }
}
