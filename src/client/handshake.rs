use super::{Client, Peer, CLIENT_ID};
use crate::{hash::Hash, torrent::Torrent};
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use serde::Serialize;
use std::net::SocketAddrV4;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const PROTOCOL_ID: u8 = 19;
const BITTORRENT: &str = "BitTorrent protocol";

#[derive(Debug, Serialize)]
struct Handshake<'hs> {
    proto_id: u8,
    proto_name: &'hs [u8],
    reserved: [u8; 8],
    info_hash: Hash,
    peer_id: &'hs [u8],
}

impl<'hs> Handshake<'hs> {
    fn new(info: Hash) -> Self {
        Self {
            proto_id: PROTOCOL_ID,
            proto_name: BITTORRENT.as_bytes(),
            reserved: [0; 8],
            info_hash: info,
            peer_id: CLIENT_ID.as_bytes(),
        }
    }

    fn to_bytes(&self) -> Bytes {
        let mut buff = BytesMut::with_capacity(68);
        buff.put_u8(self.proto_id);
        buff.put_slice(self.proto_name);
        buff.put_slice(&self.reserved);
        buff.put_slice(self.info_hash.as_bytes());
        buff.put_slice(self.peer_id);
        buff.freeze()
    }

    fn from_bytes(bytes: &'hs [u8]) -> Result<Self> {
        anyhow::ensure!(bytes.len() >= 68);
        Ok(Self {
            proto_id: bytes[0],
            proto_name: &bytes[1..=20],
            reserved: [0; 8],
            info_hash: Hash::new(bytes[28..=47].try_into()?),
            peer_id: &bytes[48..=67],
        })
    }
}

impl Client {
    pub async fn handshake(&self, p: Peer, t: &Torrent) -> Result<String> {
        let info = t.info.hash()?;
        let hs = Handshake::new(info);
        let chunk = hs.to_bytes();
        let addr: SocketAddrV4 = p.into();

        let mut stream = TcpStream::connect(addr).await?;
        stream.write_all(&chunk).await?;

        let mut buf = BytesMut::with_capacity(68);
        stream.read_buf(&mut buf).await?;

        let bytes = buf.freeze();
        let hs = Handshake::from_bytes(&bytes)?;
        Ok(hex::encode(hs.peer_id))
    }
}
