use super::{Client, Compact, Peer, CLIENT_ID};
use crate::{hash::Hash, torrent::Torrent};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::mem::size_of;

#[derive(Debug, Serialize)]
struct Request {
    #[serde(skip_serializing)]
    info_hash: Hash,
    peer_id: String,
    port: u16,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: u8,
}

impl Request {
    fn new(t: &Torrent) -> Result<Self> {
        Ok(Self {
            info_hash: t.info.hash()?,
            peer_id: CLIENT_ID.to_owned(),
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

impl Client {
    pub fn discover(&self, t: &Torrent) -> Result<Vec<Peer>> {
        let mut url = t.announce.clone();
        let req = Request::new(t)?;
        let q = req.url_encoded()?;
        url.set_query(Some(&q));
        let res = reqwest::blocking::get(url)?;
        let body = res.bytes()?;
        let res: Response = serde_bencode::from_bytes(&body)?;
        res.peers
            .chunks(size_of::<Peer>())
            .map(Peer::try_from)
            .collect()
    }
}
