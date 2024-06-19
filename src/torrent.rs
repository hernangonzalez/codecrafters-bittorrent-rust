mod info;
use anyhow::Result;
use info::Info;
use reqwest::Url;
use serde::{Deserialize, Deserializer};
use std::path::Path;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Torrent {
    #[serde(with = "url")]
    pub announce: Url,
    pub info: Info,
}

impl Torrent {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = std::fs::read(path)?;
        let t: Torrent = serde_bencode::from_bytes(&data)?;
        Ok(t)
    }
}

mod url {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let uri: Url = s.parse().map_err(serde::de::Error::custom)?;
        Ok(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open() {
        let t = Torrent::open("sample.torrent").unwrap();
        assert_eq!(
            t.announce.to_string(),
            "http://bittorrent-test-tracker.codecrafters.io/announce"
        )
    }
}
