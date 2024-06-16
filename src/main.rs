mod args;
use args::Command;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args = args::parse();
    let Some(cmd) = args.command else {
        println!("No command found.");
        return;
    };

    match cmd {
        Command::Decode { input } => dbg!(input),
    };
}
