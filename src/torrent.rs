use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Info {
    pub length: usize,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
}

impl Info {
    pub fn digest(&self) -> Result<String> {
        use sha1::{Digest, Sha1};

        let chunk = serde_bencode::to_bytes(self)?;
        let mut hasher = Sha1::new();
        hasher.update(chunk);
        let hash = hasher.finalize();
        let hex = hex::encode(hash);
        Ok(hex)
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Torrent {
    pub announce: String,
    #[serde(rename = "created by")]
    pub created_by: String,
    pub info: Info,
}

impl Torrent {
    pub fn open(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let t: Torrent = serde_bencode::from_bytes(&data)?;
        Ok(t)
    }
}
