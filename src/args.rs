use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Command {
    Decode { input: String },
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
