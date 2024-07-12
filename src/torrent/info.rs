use crate::hash::Hash;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Info {
    pub length: u32,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: u32,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>,
}

impl<'info> Info {
    pub fn hash(&self) -> Result<Hash> {
        let chunk = serde_bencode::to_bytes(self)?;
        Hash::encode(chunk)
    }

    pub fn piece_at(&self, index: usize) -> Result<Hash> {
        self.pieces()
            .enumerate()
            .find(|(i, _)| *i == index)
            .map(|e| e.1)
            .context("Piece not found at index: {index}")
    }

    pub fn pieces(&'info self) -> impl Iterator<Item = Hash> + 'info {
        Hash::build(&self.pieces)
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.len() / Hash::SIZE
    }
}
