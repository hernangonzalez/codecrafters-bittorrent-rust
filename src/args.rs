use crate::client::Peer;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Command {
    Decode { input: String },
    Info { path: PathBuf },
    Peers { path: PathBuf },
    Handshake { path: PathBuf, peer: Peer },
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

pub fn parse() -> Args {
    Args::parse()
}
