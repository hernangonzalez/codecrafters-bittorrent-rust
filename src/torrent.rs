use anyhow::Result;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::Path;

const PIECE_SIZE: usize = 20;

#[derive(PartialEq, Eq, Debug)]
struct Pieces(Vec<u8>);

impl<'de> Deserialize<'de> for Pieces {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data: Vec<u8> = serde_bytes::deserialize(deserializer)?;
        if data.len() % PIECE_SIZE != 0 {
            return Err(serde::de::Error::custom(
                "Invalid pieces length: {data.len()}",
            ));
        }
        Ok(Self(data))
    }
}

impl Serialize for Pieces {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_bytes::serialize(&self.0, serializer)
    }
}

impl Pieces {
    fn hashes(&self) -> impl Iterator<Item = &[u8]> {
        self.0.chunks(PIECE_SIZE)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Info {
    pub length: usize,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pieces: Pieces,
}

impl<'a> Info {
    pub fn digest(&self) -> Result<String> {
        use sha1::{Digest, Sha1};

        let chunk = serde_bencode::to_bytes(self)?;
        let mut hasher = Sha1::new();
        hasher.update(chunk);
        let hash = hasher.finalize();
        let hex = hex::encode(hash);
        Ok(hex)
    }

    pub fn piece_digests(&'a self) -> impl Iterator<Item = String> + 'a {
        self.pieces.hashes().map(hex::encode)
    }
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
}

impl Torrent {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path)?;
        let t: Torrent = serde_bencode::from_bytes(&data)?;
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let t = Torrent::open("sample.torrent").unwrap();
        assert_eq!(
            t.announce,
            "http://bittorrent-test-tracker.codecrafters.io/announce"
        )
    }
}
