use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::{identity, node, path::ConnectionIntent};

#[derive(Parser, Debug)]
#[command(name = "nexus", version, about = "NEXUS identity-aware secure P2P runtime prototype")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a stable root identity and a certified Noise transport key.
    Keygen { #[arg(long)] key: PathBuf },
    /// Print the stable root-identity Peer ID.
    Id { #[arg(long)] key: PathBuf },
    /// Show the transports currently implemented by this build.
    Transports,
    /// Accept one peer authenticated by a pinned stable Peer ID.
    Listen {
        #[arg(long)] key: PathBuf,
        #[arg(long, default_value = "127.0.0.1:7000")] bind: String,
        #[arg(long)] allow_peer_id: String,
    },
    /// Connect, authenticate the peer, negotiate capabilities, and send one message.
    Send {
        #[arg(long)] key: PathBuf,
        #[arg(long)] peer: String,
        #[arg(long)] peer_id: String,
        #[arg(long)] message: String,
        #[arg(long, value_enum, default_value_t = IntentArg::Balanced)] intent: IntentArg,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum IntentArg { Balanced, LowLatency, PrivacyPreferred, Reliable }

impl From<IntentArg> for ConnectionIntent {
    fn from(value: IntentArg) -> Self {
        match value {
            IntentArg::Balanced => Self::Balanced,
            IntentArg::LowLatency => Self::LowLatency,
            IntentArg::PrivacyPreferred => Self::PrivacyPreferred,
            IntentArg::Reliable => Self::Reliable,
        }
    }
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Keygen { key } => {
            let id = identity::generate(&key)?;
            println!("NEXUS 0.2 identity created.");
            println!("Key file: {}", key.display());
            println!("Stable Peer ID: {}", id.peer_id);
            println!("Root identity: Ed25519; session key: Noise X25519.");
            println!("Protect this file. It contains both private keys.");
        }
        Commands::Id { key } => println!("{}", identity::load(&key)?.peer_id),
        Commands::Transports => {
            println!("Implemented: tcp");
            println!("Planned, not implemented: quic, local-mesh, relay");
        }
        Commands::Listen { key, bind, allow_peer_id } => {
            node::listen(identity::load(&key)?, &bind, &allow_peer_id).await?;
        }
        Commands::Send { key, peer, peer_id, message, intent } => {
            node::send(identity::load(&key)?, &peer, &peer_id, &message, intent.into()).await?;
        }
    }
    Ok(())
}
