mod discover;
mod handshake;
mod peer;
pub use peer::Peer;
use serde::Serialize;

const CLIENT_ID: &str = "00112233445566778899";

#[repr(u8)]
#[derive(Debug, Serialize)]
enum Compact {
    Enabled = 1,
}

pub struct Client;
