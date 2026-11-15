use anyhow::{Context, Result};
use tokio::net::TcpStream;

use crate::path::{current_candidates, choose_path, ConnectionIntent, TransportKind};

pub trait TransportAdapter {
    fn kind(&self) -> TransportKind;
    fn name(&self) -> &'static str;
}

pub struct TcpAdapter;

impl TransportAdapter for TcpAdapter {
    fn kind(&self) -> TransportKind { TransportKind::Tcp }
    fn name(&self) -> &'static str { "tcp" }
}

pub async fn connect(peer: &str, intent: ConnectionIntent) -> Result<TcpStream> {
    let selected = choose_path(intent, &current_candidates())?;
    match selected {
        TransportKind::Tcp => TcpStream::connect(peer).await
            .with_context(|| format!("TCP adapter cannot connect to {peer}")),
        _ => anyhow::bail!("selected transport adapter is not implemented in this MVP"),
    }
}

pub fn describe_available() -> Vec<&'static str> {
    vec![TcpAdapter.name()]
}
