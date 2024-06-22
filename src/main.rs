mod args;
mod ben;
mod client;
mod hash;
mod torrent;
use anyhow::Context;
use anyhow::Result;
use args::Command;
use ben::Ben;
use client::Client;
use client::Peer;
use std::io::Write;
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
        Command::Peers { path } => handle_peers(&path).await,
        Command::Handshake { path, peer } => handle_handshake(&path, peer).await,
        Command::DownloadPiece {
            output,
            torrent,
            piece,
        } => download_piece(&output, &torrent, piece).await,
    }
}

async fn handle_peers(p: &Path) -> Result<()> {
    let client = Client::open(p)?;
    for peer in client.discover().await? {
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
    let client = Client::open(path)?;
    let stream = client.connect(peer).await?;
    println!("Peer ID: {}", stream.peer_id.digest());
    Ok(())
}

async fn download_piece(out: &Path, t: &Path, index: u32) -> Result<()> {
    let client = Client::open(t)?;
    let peers = client.discover().await?;
    let peer = peers.first().context("No peer found")?;
    let mut conn = client.connect(*peer).await?;
    let chunk = client.download_piece(&mut conn, index).await?;
    let mut file = std::fs::File::create(out)?;
    file.write_all(&chunk)?;
    println!("Piece {index} downloaded to {}.", out.display());
    Ok(())
}
