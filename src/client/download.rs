use std::cmp::min;

use super::{
    message::{
        payload::{Piece, Request},
        Code, Incoming, Outgoing,
    },
    Client, Stream,
};
use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};

pub const CHUNK_SIZE: u32 = 16 * 1024;

impl Client {
    pub async fn download_piece(&self, conn: &mut Stream, index: u32) -> Result<Bytes> {
        conn.read_code(Code::Bitfield).await?;
        conn.write_code(Code::Interested).await?;
        conn.read_code(Code::Unchoke).await?;

        let info = &self.torrrent.info;
        let mut remain = min(info.length - info.piece_length * index, info.piece_length);
        let mut begin = 0;
        let mut buffer = BytesMut::new();
        while remain > 0 {
            let len = min(remain, CHUNK_SIZE);
            let req = Request::new(index, begin, len);
            let msg = Outgoing::request(req);
            conn.write_message(&msg).await?;

            let msg: Incoming = conn.read().await?;
            let piece: Piece = msg.payload()?;
            buffer.put(piece.data);

            remain -= len;
            begin += len;
        }

        Ok(buffer.freeze())
    }
}
