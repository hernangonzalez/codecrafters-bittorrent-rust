mod announce;
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
    torrrent: Torrent,
}

impl Client {
    fn new(torrrent: Torrent) -> Self {
        let id = Hash::new(*CLIENT_ID);
        Self { id, torrrent }
    }

    pub fn open(p: &Path) -> Result<Self> {
        let t = Torrent::open(p)?;
        Ok(Self::new(t))
    }

    pub async fn discover(&self) -> Result<Vec<Peer>> {
        announce::discover(self.id, &self.torrrent).await
    }

    pub async fn connect(&self, p: Peer) -> Result<Stream> {
        Stream::open(self, p).await
    }
}
