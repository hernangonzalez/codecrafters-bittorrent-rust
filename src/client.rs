mod connect;
mod download;
mod message;
mod peer;
mod stream;
use crate::{hash::Hash, torrent::Torrent};
use anyhow::Result;
pub use peer::Peer;
use serde::Serialize;
use std::path::Path;
pub use stream::Stream;

const CLIENT_ID: &[u8; 20] = b"bittorrent-hernan-rs";

#[repr(u8)]
#[derive(Debug, Serialize)]
enum Compact {
    Enabled = 1,
}

pub struct Client {
    id: Hash,
    torrent: Torrent,
    peers: Vec<Peer>,
}

impl Client {
    fn new(torrent: Torrent) -> Self {
        let id = Hash::new(*CLIENT_ID);
        Self {
            id,
            torrent,
            peers: vec![],
        }
    }

    pub fn open(p: &Path) -> Result<Self> {
        let t = Torrent::open(p)?;
        Ok(Self::new(t))
    }
}
