use super::{
    message::{Code, Incoming, Outgoing},
    Client, Peer,
};
use crate::hash::Hash;
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use std::net::SocketAddrV4;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const PROTOCOL_ID: u8 = 19;
const PROTOCOL_NAME: &[u8] = b"BitTorrent protocol";
const RESERVED: [u8; 8] = [0u8; 8];

pub struct Stream {
    #[allow(unused)]
    stream: TcpStream,
    pub peer_id: Hash,
}

#[derive(Debug)]
struct Handshake {
    info_hash: Hash,
    peer_id: Hash,
}

impl Handshake {
    fn new(id: Hash, info: Hash) -> Self {
        Self {
            info_hash: info,
            peer_id: id,
        }
    }

    fn to_bytes(&self) -> Bytes {
        let mut buff = BytesMut::with_capacity(68);
        buff.put_u8(PROTOCOL_ID);
        buff.put_slice(PROTOCOL_NAME);
        buff.put_slice(&RESERVED);
        buff.put_slice(self.info_hash.as_bytes());
        buff.put_slice(self.peer_id.as_bytes());
        buff.freeze()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        anyhow::ensure!(bytes.len() >= 68);
        anyhow::ensure!(bytes[0] == PROTOCOL_ID);
        anyhow::ensure!(&bytes[1..20] == PROTOCOL_NAME);
        Ok(Self {
            info_hash: Hash::new(bytes[28..=47].try_into()?),
            peer_id: Hash::new(bytes[48..=67].try_into()?),
        })
    }
}

impl Stream {
    pub async fn open(c: &Client, p: Peer) -> Result<Self> {
        let info = c.torrrent.info.hash()?;
        let hs = Handshake::new(c.id, info);
        let chunk = hs.to_bytes();
        let addr: SocketAddrV4 = p.into();

        let mut stream = TcpStream::connect(addr).await?;
        stream.write_all(&chunk).await?;

        let mut buf = BytesMut::with_capacity(68);
        stream.read_buf(&mut buf).await?;

        let bytes = buf.freeze();
        let hs = Handshake::from_bytes(&bytes)?;
        Ok(Self {
            stream,
            peer_id: hs.peer_id,
        })
    }
}

pub trait Decodable: Sized {
    fn decode(bytes: &mut Bytes) -> Result<Self>;
}

pub trait Encodable: Sized {
    fn encode(&self, buf: &mut BytesMut);
    fn len(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl Stream {
    pub async fn read(&mut self) -> Result<Incoming> {
        let len = self.stream.read_u32().await?;
        let mut bytes = BytesMut::with_capacity(len as usize);
        loop {
            let read = self.stream.read_buf(&mut bytes).await?;
            if read == 0 || bytes.len() == len as usize {
                break;
            }
        }
        Incoming::decode(&mut bytes.freeze())
    }

    pub async fn read_code(&mut self, c: Code) -> Result<()> {
        let msg: Incoming = self.read().await?;
        anyhow::ensure!(msg.code == c, "Unexpected code: {} != {}", c, msg.code);
        Ok(())
    }

    async fn write(&mut self, msg: &impl Encodable) -> Result<()> {
        let mut buf = BytesMut::with_capacity(msg.len());
        self.stream.write_u32(msg.len() as u32).await?;
        msg.encode(&mut buf);
        self.stream.write_all(&buf).await?;
        Ok(())
    }

    pub async fn write_message(&mut self, out: &Outgoing<impl Encodable>) -> Result<()> {
        self.write(out).await
    }

    pub async fn write_code(&mut self, code: Code) -> Result<()> {
        self.write(&code).await
    }
}
