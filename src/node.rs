use anyhow::{Context, Result};
use serde_json::json;
use tokio::{net::TcpListener, time::{timeout, Duration}};

use crate::{
    capability,
    crypto::{self, SecureSession, receive_secure_json, send_secure_json},
    identity::Identity,
    path::ConnectionIntent,
    transport,
    wire::{Envelope, PROTOCOL_VERSION},
};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);

pub async fn listen(identity: Identity, bind: &str, allow_peer_id: &str) -> Result<()> {
    let listener = TcpListener::bind(bind).await.with_context(|| format!("cannot bind {bind}"))?;
    println!("NEXUS 0.2 listener ready: {bind}");
    println!("Local Peer ID: {}", identity.peer_id);
    println!("Allow-listed Peer ID: {allow_peer_id}");
    println!("Available transports: {}", transport::describe_available().join(", "));
    println!("Waiting for one authenticated connection...");

    let (mut socket, address) = listener.accept().await?;
    println!("Incoming TCP connection from {address}");
    let mut session = timeout(HANDSHAKE_TIMEOUT, crypto::responder(&mut socket, &identity, allow_peer_id))
        .await.context("handshake timed out")??;
    println!("Authenticated stable identity: {}", session.remote_peer_id);
    serve_one(&mut socket, &mut session).await
}

pub async fn send(identity: Identity, peer: &str, expected_peer_id: &str, message: &str, intent: ConnectionIntent) -> Result<()> {
    let mut socket = transport::connect(peer, intent).await?;
    println!("Connected to {peer} using TCP adapter");
    let mut session = timeout(HANDSHAKE_TIMEOUT, crypto::initiator(&mut socket, &identity, expected_peer_id))
        .await.context("handshake timed out")??;
    println!("Authenticated stable identity: {}", session.remote_peer_id);

    let shared = capability::negotiate(&mut socket, &mut session.transport).await?;
    println!("Negotiated capabilities: {}", shared.iter().map(|c| format!("{}@{}", c.id, c.version)).collect::<Vec<_>>().join(", "));

    send_secure_json(&mut socket, &mut session.transport, &Envelope {
        protocol: PROTOCOL_VERSION.into(),
        message_id: 2,
        kind: "data".into(),
        stream_id: 1,
        payload: json!({ "text": message }),
    }).await?;

    let ack: Envelope = receive_secure_json(&mut socket, &mut session.transport).await?;
    if ack.protocol != PROTOCOL_VERSION || ack.kind != "ack" || ack.stream_id != 1 || ack.message_id != 3 {
        anyhow::bail!("invalid ACK from peer");
    }
    println!("Encrypted ACK: {}", ack.payload);
    Ok(())
}

async fn serve_one(socket: &mut tokio::net::TcpStream, session: &mut SecureSession) -> Result<()> {
    let shared = capability::negotiate(socket, &mut session.transport).await?;
    println!("Negotiated capabilities: {}", shared.iter().map(|c| format!("{}@{}", c.id, c.version)).collect::<Vec<_>>().join(", "));

    let incoming: Envelope = receive_secure_json(socket, &mut session.transport).await?;
    if incoming.protocol != PROTOCOL_VERSION || incoming.kind != "data" || incoming.stream_id != 1 || incoming.message_id != 2 {
        anyhow::bail!("unexpected application envelope");
    }
    let message = incoming.payload.get("text").and_then(|v| v.as_str()).context("message payload missing text")?;
    println!("\n[STREAM 1] {message}");

    send_secure_json(socket, &mut session.transport, &Envelope {
        protocol: PROTOCOL_VERSION.into(),
        message_id: 3,
        kind: "ack".into(),
        stream_id: 1,
        payload: json!({ "accepted": true, "runtime": "nexus/0.2" }),
    }).await?;
    println!("Encrypted ACK sent. Session complete.");
    Ok(())
}
