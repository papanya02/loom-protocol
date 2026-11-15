mod capability;
mod cli;
mod crypto;
mod identity;
mod node;
mod path;
mod transport;
mod wire;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run(cli::Cli::parse()).await
}
