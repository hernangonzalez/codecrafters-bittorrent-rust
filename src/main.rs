mod args;
mod ben;
use anyhow::Result;
use args::Command;
use ben::Ben;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let args = args::parse();
    let Some(cmd) = args.command else {
        anyhow::bail!("No command found.");
    };

    match cmd {
        Command::Decode { input } => handle_decode(input),
    }
}

fn handle_decode(i: String) -> Result<()> {
    let ben: Ben = i.parse()?;
    println!("{ben}");
    Ok(())
}
