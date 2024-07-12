use super::{Client, Compact, Peer, Stream};
use crate::{hash::Hash, torrent::Torrent};
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

impl Client {
    pub async fn discover_peers(&mut self) -> Result<&Vec<Peer>> {
        if self.peers.is_empty() {
            self.peers = discover(self.id, &self.torrent).await?;
        }
        Ok(&self.peers)
    }

    pub async fn connect(&self, p: Peer) -> Result<Stream> {
        Stream::open(self, p).await
    }
}

async fn discover(id: Hash, t: &Torrent) -> Result<Vec<Peer>> {
    let mut url = t.announce.clone();
    let req = Request::new(id, t)?;
    let q = req.url_encoded()?;
    url.set_query(Some(&q));
    let res = reqwest::get(url).await?;
    let body = res.bytes().await?;
    let res: Response = serde_bencode::from_bytes(&body)?;
    res.peers
        .chunks(std::mem::size_of::<Peer>())
        .map(Peer::try_from)
        .collect()
}

#[derive(Debug, Serialize)]
struct Request {
    #[serde(skip_serializing)]
    info_hash: Hash,
    #[serde(with = "hash")]
    peer_id: Hash,
    port: u16,
    uploaded: u32,
    downloaded: u32,
    left: u32,
    compact: u8,
}

impl Request {
    fn new(id: Hash, t: &Torrent) -> Result<Self> {
        Ok(Self {
            info_hash: t.info.hash()?,
            peer_id: id,
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
struct Response {
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}

mod hash {
    use super::*;

    pub fn serialize<S>(input: &Hash, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_bytes::serialize(input.as_bytes(), serializer)
    }
}
