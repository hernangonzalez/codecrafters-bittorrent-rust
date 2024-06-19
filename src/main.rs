mod args;
mod ben;
mod client;
mod hash;
mod torrent;
use anyhow::Result;
use args::Command;
use ben::Ben;
use std::path::Path;
use torrent::Torrent;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let args = args::parse();
    let Some(cmd) = args.command else {
        anyhow::bail!("No command found.");
    };

    match cmd {
        Command::Decode { input } => handle_decode(input),
        Command::Info { path } => handle_info(&path),
        Command::Peers { path } => handle_peers(&path),
    }
}

fn handle_peers(p: &Path) -> Result<()> {
    let t = Torrent::open(p)?;
    for peer in client::resolve_peers(&t)? {
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
