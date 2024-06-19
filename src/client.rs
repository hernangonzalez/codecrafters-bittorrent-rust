use crate::{hash::Hash, torrent::Torrent};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, mem::size_of, net::Ipv4Addr};

const PEER_ID: &str = "00112233445566778899";

#[derive(Debug, Deserialize)]
pub struct Peer(Ipv4Addr, u16);

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

#[repr(u8)]
#[derive(Debug, Serialize)]
enum Compact {
    Enabled = 1,
}

#[derive(Debug, Serialize)]
struct PeerRequest {
    #[serde(skip_serializing)]
    info_hash: Hash,
    peer_id: String,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: u8,
}

impl PeerRequest {
    fn new(t: &Torrent) -> Result<Self> {
        Ok(Self {
            info_hash: t.info.hash()?,
            peer_id: PEER_ID.to_owned(),
            port: u16::MAX,
            uploaded: 0,
            downloaded: 0,
            left: t.info.length,
            compact: Compact::Enabled as u8,
        })
    }

    fn url_encoded(&self) -> Result<String> {
        let info_hash = self.info_hash.url_encoded();
        let q = serde_urlencoded::to_string(self)?;
        Ok(format!("{q}&info_hash={info_hash}"))
    }
}

#[derive(Debug, Deserialize)]
struct PeerResponse {
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}

pub fn resolve_peers(t: &Torrent) -> Result<Vec<Peer>> {
    let mut url = t.announce.clone();
    let req = PeerRequest::new(t)?;
    let q = req.url_encoded()?;
    url.set_query(Some(&q));
    let res = reqwest::blocking::get(url)?;
    let body = res.bytes()?;
    let res: PeerResponse = serde_bencode::from_bytes(&body)?;
    res.peers
        .chunks(size_of::<Peer>())
        .map(Peer::try_from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_display() {
        let p = Peer(Ipv4Addr::new(1, 2, 3, 4), u16::MAX);
        assert_eq!(p.to_string(), "1.2.3.4:65535")
    }
}
