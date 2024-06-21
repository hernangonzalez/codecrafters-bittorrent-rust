mod args;
mod ben;
mod client;
mod hash;
mod torrent;
use anyhow::Result;
use args::Command;
use ben::Ben;
use client::Client;
use client::Peer;
use std::path::Path;
use torrent::Torrent;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::parse();
    let Some(cmd) = args.command else {
        anyhow::bail!("No command found.");
    };

    match cmd {
        Command::Decode { input } => handle_decode(input),
        Command::Info { path } => handle_info(&path),
        Command::Peers { path } => handle_peers(&path),
        Command::Handshake { path, peer } => handle_handshake(&path, peer).await,
    }
}

fn handle_peers(p: &Path) -> Result<()> {
    let t = Torrent::open(p)?;
    let client = Client;
    for peer in client.discover(&t)? {
        println!("{peer}")
    }
    Ok(())
}

fn handle_decode(i: String) -> Result<()> {
    let ben: Ben = i.parse()?;
    println!("{ben}");
    Ok(())
}

fn handle_info(p: &Path) -> Result<()> {
    let t = Torrent::open(p)?;
    println!("Tracker URL: {}", t.announce);
    println!("Length: {}", t.info.length);
    println!("Info Hash: {}", t.info.hash()?.digest());
    println!("Piece Length: {}", t.info.piece_length);
    println!("Piece Hashes:");
    for digest in t.info.pieces().map(|p| p.digest()) {
        println!("{digest}")
    }
    Ok(())
}

async fn handle_handshake(path: &Path, peer: Peer) -> Result<()> {
    let t = Torrent::open(path)?;
    let client = Client;
    let id = client.handshake(peer, &t).await?;
    println!("Peer ID: {id}");
    Ok(())
}
