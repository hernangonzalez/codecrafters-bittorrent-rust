use anyhow::Result;
use serde::Serialize;
use sha1::{Digest, Sha1};

#[derive(Clone, Copy, Debug)]
pub struct Hash([u8; Self::SIZE]);

impl Hash {
    const SIZE: usize = 20;

    pub fn digest(&self) -> String {
        hex::encode(self.0)
    }

    pub fn url_encoded(&self) -> String {
        let dgs = self.digest();
        String::from_iter(dgs.chars().enumerate().fold(
            Vec::with_capacity(dgs.len() * 2),
            |mut acc, (i, c)| {
                if i % 2 == 0 {
                    acc.push('%');
                }
                acc.push(c);
                acc
            },
        ))
    }
}

impl<'input> Hash {
    pub fn encode<T: Serialize>(t: &T) -> Result<Self> {
        let chunk = serde_bencode::to_bytes(t)?;
        let mut hasher = Sha1::new();
        hasher.update(chunk);
        let hash = Self(hasher.finalize().into());
        Ok(hash)
    }

    pub fn build(data: &'input [u8]) -> impl Iterator<Item = Hash> + 'input {
        data.chunks(Self::SIZE).flat_map(|c| c.try_into()).map(Self)
    }
}
