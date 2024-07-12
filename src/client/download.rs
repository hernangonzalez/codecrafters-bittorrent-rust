use super::{
    message::{
        payload::{Piece, Request},
        Code, Outgoing,
    },
    Client, Stream,
};
use crate::{hash::Hash, torrent::Torrent};
use anyhow::{ensure, Context, Ok, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::{cmp::min, fs::File, io::Read, path::Path};
use std::{io::Write, path::PathBuf};

pub const CHUNK_SIZE: u32 = 16 * 1024;

struct Fetch {
    hash: Hash,
    parts: Vec<Request>,
}

impl<'a> Torrent {
    fn fetch_index(&self, index: u32) -> Result<Fetch> {
        let hash = self.info.piece_at(index as usize)?;
        let info = &self.info;
        let mut remains = min(info.length - info.piece_length * index, info.piece_length);
        let mut parts = Vec::with_capacity(self.info.piece_count());
        while remains > 0 {
            let len = min(remains, CHUNK_SIZE);
            let req = Request::new(index, parts.len() as u32 * CHUNK_SIZE, len);
            parts.push(req);
            remains -= len;
        }
        Ok(Fetch { hash, parts })
    }

    fn fetch_all(&'a self) -> impl Iterator<Item = Fetch> + 'a {
        (0..self.info.piece_count() as u32).flat_map(|i| self.fetch_index(i))
    }
}

impl Stream {
    async fn fetch(&mut self, req: Request) -> Result<Piece> {
        let out = Outgoing::request(req);
        self.write_message(&out).await?;
        let inc = self.read().await?;
        inc.payload()
    }

    async fn download(&mut self, fetch: Fetch) -> Result<Bytes> {
        self.read_code(Code::Bitfield).await?;
        self.write_code(Code::Interested).await?;
        self.read_code(Code::Unchoke).await?;

        let mut buffer = BytesMut::new();
        for r in fetch.parts {
            let piece = self.fetch(r).await?;
            buffer.put(piece.data);
        }

        let hash = Hash::encode(&buffer)?;
        ensure!(
            hash == fetch.hash,
            "Downloaded piece does not match expected hash. {} - {}",
            fetch.hash,
            hash
        );

        Ok(buffer.freeze())
    }
}

impl Client {
    async fn deque_stream(&mut self) -> Result<Stream> {
        self.discover_peers().await?;
        let peer = *self.peers.first().context("No peers available")?;
        self.connect(peer).await
    }

    pub async fn download_piece(&mut self, index: u32, out: &Path) -> Result<()> {
        let req = self.torrent.fetch_index(index)?;
        let mut conn = self.deque_stream().await?;
        let chunk = conn.download(req).await?;
        let mut file = File::create(out)?;
        file.write_all(&chunk)?;
        Ok(())
    }

    fn collect_parts(&self, paths: &[PathBuf], out: &Path) -> Result<()> {
        let mut out = File::create(out)?;
        for path in paths {
            let mut f = File::open(path)?;
            let mut buffer = Vec::with_capacity(self.torrent.info.piece_length as usize);
            f.read_to_end(&mut buffer)?;
            out.write_all(&buffer)?;
            drop(f);
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub async fn download(&mut self, out: &Path) -> Result<()> {
        let pieces: Vec<_> = self.torrent.fetch_all().collect();
        let mut parts = vec![];
        for p in pieces {
            let out = out.with_extension(p.hash.digest());
            let mut conn = self.deque_stream().await?;
            let chunk = conn.download(p).await?;
            let mut file = std::fs::File::create(&out)?;
            file.write_all(&chunk)?;
            parts.push(out);
        }
        self.collect_parts(&parts, out)
    }
}
