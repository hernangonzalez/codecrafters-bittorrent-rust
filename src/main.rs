mod args;
mod ben;
mod decode;
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
    }
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
    println!("Info Hash: {}", t.info.digest()?);
    Ok(())
}
