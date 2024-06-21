use anyhow::Result;
use serde::Serialize;
use sha1::{Digest, Sha1};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Hash([u8; Self::SIZE]);

impl Hash {
    const SIZE: usize = 20;

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

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
    pub fn new(b: [u8; 20]) -> Self {
        Self(b)
    }

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

#[cfg(test)]
mod tests {
    use super::Hash;

    const CHUNK: &[u8] = &[
        0xE8, 0x76, 0xF6, 0x7A, 0x2A, 0x88, 0x86, 0xE8, 0xF3, 0x6B, 0x13, 0x67, 0x26, 0xC3, 0x0F,
        0xA2, 0x97, 0x03, 0x02, 0x2D, 0x6E, 0x22, 0x75, 0xE6, 0x04, 0xA0, 0x76, 0x66, 0x56, 0x73,
        0x6E, 0x81, 0xFF, 0x10, 0xB5, 0x52, 0x04, 0xAD, 0x8D, 0x35, 0xF0, 0x0D, 0x93, 0x7A, 0x02,
        0x13, 0xDF, 0x19, 0x82, 0xBC, 0x8D, 0x09, 0x72, 0x27, 0xAD, 0x9E, 0x90, 0x9A, 0xCC, 0x17,
        0x65, 0x65,
    ];

    #[test]
    fn test_build() {
        let hashes: Vec<_> = Hash::build(CHUNK).collect();
        assert_eq!(hashes.len(), 3)
    }

    #[test]
    fn test_url_encoded() {
        let hash = Hash::build(CHUNK).next().unwrap();
        let s = hash.url_encoded();
        assert_eq!(
            s,
            "%e8%76%f6%7a%2a%88%86%e8%f3%6b%13%67%26%c3%0f%a2%97%03%02%2d"
        );
    }

    #[test]
    fn test_digest() {
        let hash = Hash::build(CHUNK).next().unwrap();
        let d = hash.digest();
        assert_eq!(d, "e876f67a2a8886e8f36b136726c30fa29703022d");
    }
}
