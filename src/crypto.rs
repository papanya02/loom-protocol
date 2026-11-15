use anyhow::{bail, Context, Result};
use snow::{params::NoiseParams, Builder, TransportState};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    identity::{self, Identity, KeyCertificate},
    wire::{read_record, write_record, MAX_PLAINTEXT},
};

const PATTERN: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
const PROLOGUE: &[u8] = b"NEXUS/0.2;identity-cert-v1;capability-v1";

pub struct SecureSession {
    pub transport: TransportState,
    pub remote_peer_id: String,
}

pub async fn initiator<S>(stream: &mut S, local: &Identity, expected_peer_id: &str) -> Result<SecureSession>
where S: AsyncRead + AsyncWrite + Unpin {
    let params: NoiseParams = PATTERN.parse()?;
    let mut hs = Builder::new(params)
        .local_private_key(&local.noise_private_key)?
        .prologue(PROLOGUE)?
        .build_initiator()?;

    let mut out = vec![0u8; MAX_PLAINTEXT];
    let n = hs.write_message(&[], &mut out)?;
    write_record(stream, &out[..n]).await?;

    let incoming = read_record(stream).await?;
    let mut payload = vec![0u8; MAX_PLAINTEXT];
    hs.read_message(&incoming, &mut payload)?;

    let n = hs.write_message(&[], &mut out)?;
    write_record(stream, &out[..n]).await?;
    finish(stream, hs, local, expected_peer_id).await
}

pub async fn responder<S>(stream: &mut S, local: &Identity, expected_peer_id: &str) -> Result<SecureSession>
where S: AsyncRead + AsyncWrite + Unpin {
    let params: NoiseParams = PATTERN.parse()?;
    let mut hs = Builder::new(params)
        .local_private_key(&local.noise_private_key)?
        .prologue(PROLOGUE)?
        .build_responder()?;

    let incoming = read_record(stream).await?;
    let mut payload = vec![0u8; MAX_PLAINTEXT];
    hs.read_message(&incoming, &mut payload)?;

    let mut out = vec![0u8; MAX_PLAINTEXT];
    let n = hs.write_message(&[], &mut out)?;
    write_record(stream, &out[..n]).await?;

    let incoming = read_record(stream).await?;
    hs.read_message(&incoming, &mut payload)?;
    finish(stream, hs, local, expected_peer_id).await
}

async fn finish<S>(stream: &mut S, hs: snow::HandshakeState, local: &Identity, expected_peer_id: &str) -> Result<SecureSession>
where S: AsyncRead + AsyncWrite + Unpin {
    if !hs.is_handshake_finished() {
        bail!("Noise handshake did not finish");
    }
    let remote_static = hs.get_remote_static().context("remote Noise static key missing")?.to_vec();
    let mut transport = hs.into_transport_mode().context("cannot enter Noise transport mode")?;

    // Exchange root-signed certificates inside the encrypted Noise channel.
    // Both sides send before reading, so neither waits for the other to initiate.
    let local_cert = local.certificate();
    send_secure_json(stream, &mut transport, &local_cert).await?;
    let remote_cert: KeyCertificate = receive_secure_json(stream, &mut transport).await?;
    let remote_peer_id = identity::verify_certificate(&remote_cert, &remote_static)?;

    if !remote_peer_id.eq_ignore_ascii_case(expected_peer_id) {
        bail!("peer identity mismatch: expected {}, received {}", expected_peer_id, remote_peer_id);
    }
    Ok(SecureSession { transport, remote_peer_id })
}

pub async fn send_secure_json<S, T>(stream: &mut S, transport: &mut TransportState, value: &T) -> Result<()>
where S: AsyncRead + AsyncWrite + Unpin, T: serde::Serialize {
    let plaintext = serde_json::to_vec(value)?;
    if plaintext.len() > MAX_PLAINTEXT { bail!("encrypted payload too large"); }
    let mut ciphertext = vec![0u8; MAX_PLAINTEXT + 64];
    let n = transport.write_message(&plaintext, &mut ciphertext)?;
    write_record(stream, &ciphertext[..n]).await
}

pub async fn receive_secure_json<S, T>(stream: &mut S, transport: &mut TransportState) -> Result<T>
where S: AsyncRead + AsyncWrite + Unpin, T: serde::de::DeserializeOwned {
    let ciphertext = read_record(stream).await?;
    let mut plaintext = vec![0u8; MAX_PLAINTEXT];
    let n = transport.read_message(&ciphertext, &mut plaintext)?;
    serde_json::from_slice(&plaintext[..n]).context("invalid encrypted JSON payload")
}
