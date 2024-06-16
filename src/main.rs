mod args;
mod decode;
use anyhow::Result;
use args::Command;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<()> {
    let args = args::parse();
    let Some(cmd) = args.command else {
        anyhow::bail!("No command found.");
    };

    match cmd {
        Command::Decode { input } => handle_decode(&input),
    }
}

fn handle_decode(i: &str) -> Result<()> {
    let Some(c) = i.chars().next() else {
        anyhow::bail!("Input is empty");
    };

    if c == 'i' {
        let i = decode::integer(i)?;
        println!("{i}");
    } else if c.is_ascii_digit() {
        let s = decode::string(i)?;
        println!("{s}");
    } else {
        anyhow::bail!("Unknown encoded argument: {i}");
    }

    Ok(())
}
