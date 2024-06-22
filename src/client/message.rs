use super::stream::{Decodable, Encodable};
use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::Display;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Code {
    Interested = 2,
    Bitfield = 5,
    Unchoke = 1,
    Request = 6,
    Piece = 7,
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (*self as u32).fmt(f)
    }
}

impl Decodable for Code {
    fn decode(bytes: &mut Bytes) -> Result<Self> {
        anyhow::ensure!(bytes.len() >= std::mem::size_of::<Self>());
        match bytes.get_u8() {
            c if c == Self::Bitfield as u8 => Ok(Self::Bitfield),
            c if c == Self::Interested as u8 => Ok(Self::Interested),
            c if c == Self::Unchoke as u8 => Ok(Self::Unchoke),
            c if c == Self::Request as u8 => Ok(Self::Request),
            c if c == Self::Piece as u8 => Ok(Self::Piece),
            other => anyhow::bail!("Not a valid code: {other}"),
        }
    }
}

impl Encodable for Code {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(*self as u8)
    }
}

#[derive(Debug)]
pub struct Incoming {
    pub code: Code,
    data: Bytes,
}

impl Decodable for Incoming {
    fn decode(bytes: &mut Bytes) -> Result<Self> {
        let code = Code::decode(bytes)?;
        let data = bytes.slice(..bytes.remaining());
        Ok(Self { code, data })
    }
}

impl Incoming {
    pub fn payload<T: Decodable>(&self) -> Result<T> {
        T::decode(&mut self.data.clone())
    }
}

#[derive(Debug)]
pub struct Outgoing<Payload> {
    code: Code,
    data: Payload,
}

impl Outgoing<payload::Request> {
    pub fn request(data: payload::Request) -> Self {
        Self {
            code: Code::Request,
            data,
        }
    }
}

impl<T> Encodable for Outgoing<T>
where
    T: Encodable,
{
    fn encode(&self, buf: &mut BytesMut) {
        self.code.encode(buf);
        self.data.encode(buf);
    }
}

pub mod payload {
    use super::*;

    #[derive(Debug)]
    pub struct Request {
        pub index: u32,
        pub begin: u32,
        pub length: u32,
    }

    impl Request {
        pub fn new(index: u32, begin: u32, length: u32) -> Self {
            Self {
                index,
                begin,
                length,
            }
        }
    }

    impl Encodable for Request {
        fn encode(&self, buf: &mut BytesMut) {
            buf.put_u32(self.index);
            buf.put_u32(self.begin);
            buf.put_u32(self.length);
        }
    }

    #[allow(dead_code)]
    #[derive(Debug)]
    pub struct Piece {
        pub index: u32,
        pub begin: u32,
        pub data: Bytes,
    }

    impl Decodable for Piece {
        fn decode(bytes: &mut Bytes) -> Result<Self> {
            anyhow::ensure!(bytes.len() >= 8);
            let piece = Self {
                index: bytes.get_u32(),
                begin: bytes.get_u32(),
                data: bytes.clone(),
            };
            bytes.clear();
            Ok(piece)
        }
    }
}
